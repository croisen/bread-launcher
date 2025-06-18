use std::sync::Arc;

use serde::de::Deserializer;
use serde::ser::Serializer;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex as TKMutex;

pub fn serialize<S: Serializer, T: Serialize>(
    val: &Arc<TKMutex<T>>,
    s: S,
) -> Result<S::Ok, S::Error> {
    T::serialize(&val.blocking_lock(), s)
}

pub fn deserialize<'de, D: Deserializer<'de>, T: Deserialize<'de>>(
    d: D,
) -> Result<Arc<TKMutex<T>>, D::Error> {
    Ok(Arc::new(TKMutex::new(T::deserialize(d)?)))
}
