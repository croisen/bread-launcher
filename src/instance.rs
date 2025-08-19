use std::cmp::PartialEq;
use std::collections::BTreeMap;
use std::fs::create_dir_all;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpmc::{Receiver as MultiReceiver, Sender as MultiSender, channel as multi_channel};
use std::sync::mpsc::Sender as SingleSender;
use std::thread::{JoinHandle, sleep, spawn};
use std::time::Duration;

use anyhow::{Result, bail};
use parking_lot::Mutex;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

use crate::account::Account;
use crate::init::UNGROUPED_NAME;
use crate::minecraft::{Minecraft, MinecraftVersion};
use crate::utils::message::Message;

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct Instances {
    // Group name, Instance Name, Instance
    col: BTreeMap<String, BTreeMap<String, Arc<Mutex<Instance>>>>,
}

impl Instances {
    pub fn new() -> Self {
        Self {
            col: BTreeMap::new(),
        }
    }

    pub fn add_instance(&mut self, group_name: impl AsRef<str>, instance: Instance) {
        let group_name = if group_name.as_ref().is_empty() {
            UNGROUPED_NAME.to_string()
        } else {
            group_name.as_ref().to_string()
        };

        if let Some(instances) = self.col.get_mut::<str>(group_name.as_ref()) {
            instances.insert(
                instance.name.as_ref().to_string(),
                Mutex::new(instance).into(),
            );
        } else {
            let mut instances = BTreeMap::new();
            instances.insert(
                instance.name.as_ref().to_string(),
                Mutex::new(instance).into(),
            );
            self.col.insert(group_name, instances);
        };
    }

    pub fn get_instances(
        &mut self,
    ) -> &mut BTreeMap<String, BTreeMap<String, Arc<Mutex<Instance>>>> {
        &mut self.col
    }

    pub fn new_vanilla_instance(
        cl: Client,
        name: impl AsRef<str>,
        mc_ver: impl AsRef<str>,
        full_ver: impl AsRef<str>,
        version: Arc<MinecraftVersion>,
    ) -> Result<Instance> {
        version.download(cl)?;
        let m = Minecraft::new(Path::new("a"), &mc_ver)?;
        let c = m.new_instance()?;
        let instance = Instance::new(
            name,
            mc_ver,
            full_ver,
            c.get_cache_dir(),
            InstanceLoader::Vanilla,
        );

        Ok(instance)
    }
}

#[derive(Default, Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub enum InstanceLoader {
    #[default]
    Vanilla = 0,
    Forge = 1,
    Fabric = 2,
    LiteLoader = 3,
    Quilt = 4,
}

impl From<InstanceLoader> for usize {
    fn from(value: InstanceLoader) -> Self {
        value as Self
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Instance {
    pub name: Arc<str>,
    pub mc_ver: Arc<str>,
    pub full_ver: Arc<str>, // Getting ready for different mod loader versions
    pub path: Arc<PathBuf>,
    pub loader: InstanceLoader,

    #[serde(skip)]
    run: Option<JoinHandle<()>>,
    #[serde(skip, default = "multi_channel::<()>")]
    channel: (MultiSender<()>, MultiReceiver<()>),
}

impl Instance {
    pub fn new(
        name: impl AsRef<str>,
        mc_ver: impl AsRef<str>,
        full_ver: impl AsRef<str>,
        path: Arc<PathBuf>,
        loader: InstanceLoader,
    ) -> Self {
        Self {
            name: name.as_ref().into(),
            mc_ver: mc_ver.as_ref().into(),
            full_ver: full_ver.as_ref().into(),
            path,
            loader,
            run: None,
            channel: multi_channel::<()>(),
        }
    }

    pub fn run_offline(
        &mut self,
        cl: Client,
        steps: (Arc<AtomicUsize>, Arc<AtomicUsize>),
        tx: SingleSender<Message>,
        ram: usize,
        account: Arc<Mutex<Account>>,
    ) -> Result<()> {
        create_dir_all(self.path.as_ref())?;
        let account = account.lock().clone().into();
        let name = self.name.clone();
        let mc_ver = self.mc_ver.clone();
        let full_ver = self.full_ver.clone();
        let path = self.path.clone();
        let loader = self.loader;

        let stop_rx = self.channel.1.clone();
        self.run = Some(spawn(move || {
            let run = Self::__run(
                cl,
                (name, mc_ver, full_ver, path, loader),
                steps.clone(),
                tx.clone(),
                stop_rx,
                ram,
                account,
            );

            if let Err(e) = run {
                log::error!("Error in launching instance: {e:?}");
                let _ = tx.send(Message::errored(format!(
                    "Error in launching instance: {e}"
                )));
            }
        }));

        Ok(())
    }

    pub fn run(
        &mut self,
        cl: Client,
        steps: (Arc<AtomicUsize>, Arc<AtomicUsize>),
        tx: SingleSender<Message>,
        ram: usize,
        account: Arc<Mutex<Account>>,
    ) -> Result<()> {
        create_dir_all(self.path.as_ref())?;
        let account: Arc<Account> = account.lock().clone().into();
        let name = self.name.clone();
        let mc_ver = self.mc_ver.clone();
        let full_ver = self.full_ver.clone();
        let path = self.path.clone();
        let loader = self.loader;

        let stop_rx = self.channel.1.clone();
        self.run = Some(spawn(move || {
            let download = Self::__download(
                cl.clone(),
                (
                    name.clone(),
                    mc_ver.clone(),
                    full_ver.clone(),
                    path.clone(),
                    loader,
                ),
                steps.clone(),
                tx.clone(),
                stop_rx.clone(),
                ram,
                account.clone(),
            );

            if let Err(e) = download {
                log::error!("Error in launching instance: {e:?}");
                let _ = tx.send(Message::errored(format!(
                    "Error in launching instance: {e}"
                )));

                return;
            }

            if !download.unwrap() {
                // Eat the remaning stop requests
                let _ = tx.send(Message::errored("Stop signal received"));
                while stop_rx.try_recv().is_ok() {}
                return;
            }

            let run = Self::__run(
                cl,
                (name, mc_ver, full_ver, path, loader),
                steps.clone(),
                tx.clone(),
                stop_rx,
                ram,
                account,
            );

            if let Err(e) = run {
                log::error!("Error in launching instance: {e:?}");
                let _ = tx.send(Message::errored(format!(
                    "Error in launching instance: {e}"
                )));
            }
        }));

        Ok(())
    }

    pub fn is_running(&self) -> bool {
        if let Some(run) = &self.run {
            return !run.is_finished();
        }

        false
    }

    pub fn stop(&mut self) {
        let _ = self.channel.0.send(());
        let _ = self.run.take();
    }

    fn __download(
        cl: Client,
        instance: (Arc<str>, Arc<str>, Arc<str>, Arc<PathBuf>, InstanceLoader),
        steps: (Arc<AtomicUsize>, Arc<AtomicUsize>),
        tx: SingleSender<Message>,
        rx: MultiReceiver<()>,
        _ram: usize,
        _account: Arc<Account>,
    ) -> Result<bool> {
        let (_name, mc_ver, _full_ver, path, loader) = instance;
        let m = Minecraft::new(path.as_ref(), mc_ver.as_ref())?;
        if !m.download_jre(cl.clone(), steps.clone(), tx.clone(), rx.clone())? {
            return Ok(false);
        }

        if !m.download_client(cl.clone(), steps.clone(), tx.clone(), rx.clone())? {
            return Ok(false);
        }

        if !m.download_assets(cl.clone(), steps.clone(), tx.clone(), rx.clone())? {
            return Ok(false);
        }

        match loader {
            InstanceLoader::Vanilla => {}
            _ => bail!("Unimplemented"),
        }

        if rx.try_recv().is_ok() {
            let _ = tx.send(Message::errored("Stop signal received"));
            return Ok(false);
        }

        Ok(true)
    }

    fn __run(
        _cl: Client,
        instance: (Arc<str>, Arc<str>, Arc<str>, Arc<PathBuf>, InstanceLoader),
        steps: (Arc<AtomicUsize>, Arc<AtomicUsize>),
        tx: SingleSender<Message>,
        rx: MultiReceiver<()>,
        ram: usize,
        account: Arc<Account>,
    ) -> Result<()> {
        let (name, mc_ver, _full_ver, path, loader) = instance;
        let _ = tx.send(Message::msg(format!("Now launching instance {name}")));
        steps.0.store(1, Ordering::SeqCst);
        steps.1.store(1, Ordering::SeqCst);
        let m = Minecraft::new(path.as_ref(), mc_ver.as_ref())?;
        let mut child = match loader {
            InstanceLoader::Vanilla => m.run(ram, account)?,
            _ => bail!("Unimplemented"),
        };

        loop {
            sleep(Duration::from_secs(1));
            if rx.try_recv().is_ok() {
                let _ = tx.send(Message::errored("Stop signal received"));
                break;
            }

            let wait = child.try_wait();
            if wait.is_err() {
                bail!(wait.unwrap_err());
            }

            if let Some(status) = wait.unwrap() {
                let _ = tx.send(Message::msg(format!(
                    "Instance {name} exited with status {status:?}"
                )));

                break;
            }
        }

        child.kill()?;
        // Eat the remaning stop requests
        while rx.try_recv().is_ok() {}

        Ok(())
    }
}

impl Default for Instance {
    fn default() -> Self {
        Self {
            name: Arc::default(),
            mc_ver: Arc::default(),
            full_ver: Arc::default(),
            path: Arc::default(),
            loader: InstanceLoader::Vanilla,
            run: None,
            channel: multi_channel::<()>(),
        }
    }
}

impl PartialEq for Instance {
    fn eq(&self, other: &Self) -> bool {
        let a = self.name == other.name;
        let b = self.mc_ver == other.mc_ver;
        let c = self.full_ver == other.full_ver;
        let d = self.loader == other.loader;
        let e = self.path == other.path;

        a && b && c && d && e
    }
}
