use egui::{
    Color32, Image, Pos2, Rect, Response, Sense, StrokeKind, TextStyle, TextWrapMode,
    TextureHandle, Ui, Vec2, Widget, WidgetText,
};

pub struct SelectableImageLabel<'a> {
    img: Image<'a>,
    max_img_size: Vec2,
    label: WidgetText,
    selected: bool,
}

impl<'a> SelectableImageLabel<'a> {
    pub fn new(
        selected: bool,
        src: TextureHandle,
        max_img_size: Vec2,
        text: impl Into<WidgetText>,
    ) -> Self {
        Self {
            img: Image::new((src.id(), src.size_vec2())).max_size(max_img_size),
            max_img_size,
            label: text.into(),
            selected,
        }
    }
}

impl<'a> Widget for SelectableImageLabel<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let tgalley = self.label.into_galley(
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

        let (rect, res) = ui.allocate_exact_size(size, Sense::click());
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

        if ui.is_rect_visible(rect) {
            let (rounding, fill, stroke) = if self.selected {
                let selection = ui.visuals().selection;
                (
                    self.img.image_options().corner_radius,
                    selection.bg_fill,
                    selection.stroke,
                )
            } else {
                let visuals = ui.style().interact(&res);
                (
                    self.img.image_options().corner_radius,
                    visuals.weak_bg_fill,
                    visuals.bg_stroke,
                )
            };

            ui.painter().rect_filled(rect, rounding, fill);
            self.img.paint_at(ui, img_rect);
            ui.painter().galley(text_rect.min, tgalley, Color32::WHITE);
            ui.painter()
                .rect_stroke(rect, rounding, stroke, StrokeKind::Inside);
        }

        res
    }
}
