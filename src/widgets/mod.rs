use egui::{
    Color32, Image, ImageSource, Pos2, Rect, Response, Sense, StrokeKind, TextStyle, TextWrapMode,
    Ui, Vec2, WidgetText,
};

/// Used for instance icons
pub fn selectable_image_label<T: Sized + PartialEq>(
    ui: &mut Ui,
    source: &ImageSource,
    text: impl Into<WidgetText>,
    current_value: &mut T,
    selected_value: T,
) -> Response {
    let text: WidgetText = text.into();
    let tgalley = text.into_galley(
        ui,
        Some(TextWrapMode::Truncate),
        50.0,
        TextStyle::Button.resolve(ui.style()),
    );

    let text_size = tgalley.rect.max;
    // Turns out the button padding is only for the sides
    let padding = Vec2::splat(ui.style().spacing.button_padding.x);
    let max_img_size = Vec2 { x: 50.0, y: 50.0 };
    let mut size = 2.0 * padding + max_img_size;
    size.y += text_size.y + padding.y;

    let img = Image::new(source.to_owned());
    let (rect, mut res) = ui.allocate_exact_size(size, Sense::click());
    let img_rect = Rect {
        min: rect.min + padding,
        max: Pos2 {
            x: rect.right() - padding.x,
            y: rect.bottom() - padding.y * 2.0 - text_size.y,
        },
    };
    let text_rect = Rect {
        min: Pos2 {
            x: rect.left() + padding.x,
            y: rect.top() + 50.0 + padding.y,
        },
        max: rect.max - padding,
    };

    let selected = *current_value == selected_value;
    if ui.is_rect_visible(rect) {
        let (rounding, fill, stroke) = if selected {
            let selection = ui.visuals().selection;
            (
                img.image_options().corner_radius,
                selection.bg_fill,
                selection.stroke,
            )
        } else {
            let visuals = ui.style().interact(&res);
            (
                img.image_options().corner_radius,
                visuals.weak_bg_fill,
                visuals.bg_stroke,
            )
        };

        ui.painter().rect_filled(rect, rounding, fill);
        ui.put(img_rect, img);
        ui.painter().galley(text_rect.min, tgalley, Color32::WHITE);
        ui.painter()
            .rect_stroke(rect, rounding, stroke, StrokeKind::Inside);
    }

    if res.clicked() && !selected {
        *current_value = selected_value;
        res.mark_changed();
    }

    res
}
