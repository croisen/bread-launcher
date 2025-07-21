use egui::{Image, ImageButton, Label, Response, Ui, Vec2, WidgetText};

pub fn selectable_image_label<T: Sized + PartialEq>(
    ui: &mut Ui,
    source: impl AsRef<str>,
    text: impl Into<WidgetText>,
    current_value: &mut T,
    selected_value: T,
) -> Response {
    let image = Image::from_uri(source.as_ref()).max_size(Vec2 { x: 50.0, y: 50.0 });
    let mut r = ui.add(ImageButton::new(image));
    ui.put(r.rect, Label::new(text.into()).wrap());
    if r.clicked() && *current_value != selected_value {
        *current_value = selected_value;
        r.mark_changed();
    }

    r
}
