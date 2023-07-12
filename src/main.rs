mod gestures;

use egui::{vec2, Align2, FontId, Pos2, Sense, Shape};
use gestures::gesture_from_positions;

#[derive(Default)]
struct GesturesApp {
    gesture_path: Option<Vec<Pos2>>,
    gesture_name: Option<String>,
}

impl eframe::App for GesturesApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Gestures");
            ui.separator();

            ui.scope(|ui| {
                let (rect, response) = ui.allocate_exact_size(vec2(512.0, 512.0), Sense::click_and_drag());

                if response.drag_started() {
                    self.gesture_path = Some(vec![response.interact_pointer_pos().unwrap()]);
                }

                if response.dragged() {
                    if let Some(gesture_path) = &mut self.gesture_path {
                        gesture_path.push(response.interact_pointer_pos().unwrap())
                    }
                }

                if response.drag_released() {
                    if let Some(gesture_path) = &self.gesture_path {
                        let positions = gesture_path.iter().map(<(f32, f32)>::from).collect::<Vec<_>>();
                        self.gesture_name = gesture_from_positions(&positions).map(String::from);
                    }
                }

                let style = *ui.style().interact(&response);

                ui.painter()
                    .rect(rect, style.rounding, ui.visuals().extreme_bg_color, style.bg_stroke);

                let font_id = FontId::new(128.0, egui::FontFamily::Proportional);
                ui.painter().text(
                    rect.center(),
                    Align2::CENTER_CENTER,
                    "\u{1F58A}",
                    font_id,
                    ui.visuals().faint_bg_color,
                );

                if let Some(gesture_path) = &mut self.gesture_path {
                    ui.set_clip_rect(rect);
                    ui.painter().add(Shape::line(gesture_path.to_vec(), style.fg_stroke));

                    ui.painter().circle(
                        *gesture_path.first().unwrap(),
                        4.0,
                        ui.visuals().extreme_bg_color,
                        style.fg_stroke,
                    );

                    ui.painter().circle(
                        *gesture_path.last().unwrap(),
                        4.0,
                        style.fg_stroke.color,
                        style.fg_stroke,
                    );
                }
            });

            if let Some(gesture_name) = &self.gesture_name {
                ui.separator();
                ui.heading(gesture_name);
            }
        });
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        initial_window_size: Some(vec2(550.0, 600.0)),
        ..Default::default()
    };

    eframe::run_native("Gestures", options, Box::new(|_| Box::<GesturesApp>::default()))
}
