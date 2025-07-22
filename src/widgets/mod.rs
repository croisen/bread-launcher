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
    let tgalley = text.text_style(egui::TextStyle::Button).into_galley(
        ui,
        Some(TextWrapMode::Wrap),
        50.0,
        TextStyle::Button.resolve(ui.style()),
    );

    let text_size = tgalley.rect.max;
    let padding = ui.style().spacing.button_padding;
    let max_img_size = Vec2 { x: 50.0, y: 50.0 };
    let mut size = 2.0 * padding + max_img_size;
    size.y += text_size.y + 1.0;

    let img = Image::new(source.to_owned());
    let (rect, mut res) = ui.allocate_exact_size(size, Sense::click());
    let img_rect = rect.shrink2(Vec2 {
        x: 0.0,
        y: text_size.y,
    });
    let text_rect = Rect {
        min: Pos2 {
            x: rect.left(),
            y: rect.top() + 51.0,
        },
        max: Pos2 {
            x: rect.right(),
            y: rect.bottom(),
        },
    };

    let selected = *current_value == selected_value;
    if ui.is_rect_visible(rect) {
        let (expansion, rounding, fill, stroke) = if selected {
            let selection = ui.visuals().selection;
            (
                Vec2::ZERO,
                img.image_options().corner_radius,
                selection.bg_fill,
                selection.stroke,
            )
        } else {
            let visuals = ui.style().interact(&res);
            let expansion = Vec2::splat(visuals.expansion);
            (
                expansion,
                img.image_options().corner_radius,
                visuals.weak_bg_fill,
                visuals.bg_stroke,
            )
        };

        ui.painter()
            .rect_filled(rect.expand2(expansion), rounding, fill);

        ui.put(img_rect, img);
        ui.painter()
            .galley(text_rect.min + padding, tgalley, Color32::WHITE);

        ui.painter().rect_stroke(
            rect.expand2(expansion),
            rounding,
            stroke,
            StrokeKind::Inside,
        );
    }

    if res.clicked() && !selected {
        *current_value = selected_value;
        res.mark_changed();
    }

    res
}
