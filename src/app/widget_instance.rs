use std::sync::Arc;

use egui::widgets::ImageButton;
use egui::{Image, ImageSource, Response, Ui, Vec2};

use crate::instance::Instance;

pub fn widget_instance_button(
    ui: &mut Ui,
    sel_instance: &mut Arc<Instance>,
    selected: &mut bool,
    instance: Arc<Instance>,
    img_src: ImageSource,
    text: &str,
) -> Response {
    ui.vertical(|ui| {
        let sel = *sel_instance == instance;
        let mut response = ui.add(
            ImageButton::new(Image::new(img_src).fit_to_exact_size(Vec2 { x: 100.0, y: 100.0 }))
                .frame(false)
                .selected(sel),
        );

        if response.clicked() && !sel {
            *sel_instance = instance;
            *selected = true;
            response.mark_changed();
        };

        response.labelled_by(ui.label(text).id)
    })
    .response
}
