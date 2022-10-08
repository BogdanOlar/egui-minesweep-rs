use eframe::{egui::{PointerButton, self, Layout, Label, RichText, Button, Context, TextStyle, Ui, CentralPanel, Sense, Direction}, epaint::Color32, Frame, NativeOptions, App};
use egui_extras::{TableBuilder, Size};
use minefield::{Minefield, SpotState, StepResult, SpotKind};

mod minefield;

fn main() {
    let options = NativeOptions::default();
    eframe::run_native(
        "Minesweep-Rs",
        options,
        Box::new(|_cc| Box::new(MinesweepRsApp::default())),
    );
}

struct MinesweepRsApp {
    minefield: Minefield,
    game_state: GameState,
}

impl MinesweepRsApp {
    fn render_top_panel(&mut self, ctx: &Context, frame: &mut Frame) {
        // define a TopBottomPanel widget
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.add_space(10.);
            egui::menu::bar(ui, |ui| {
                // logo
                ui.with_layout(Layout::left_to_right(egui::Align::TOP), |ui| {
                    ui.add(Label::new(
                        RichText::new("📰").text_style(egui::TextStyle::Heading),
                    ));
                });
                // controls
                ui.with_layout(Layout::right_to_left(egui::Align::TOP), |ui| {
                    if !cfg!(target_arch = "wasm32") {
                        let close_btn = ui.add(Button::new(
                            RichText::new("❌").text_style(egui::TextStyle::Body),
                        ));
                        if close_btn.clicked() {
                            frame.quit();
                        }
                    }
                    let refresh_btn = ui.add(Button::new(
                        RichText::new("🔄").text_style(egui::TextStyle::Body),
                    ));
                    
                    if refresh_btn.clicked() {
                        // TODO: asd    
                    }

                    // theme button
                    // let theme_btn = ui.add(Button::new(
                    //     RichText::new({
                    //         if self.config.dark_mode {
                    //             "🌞"
                    //         } else {
                    //             "🌙"
                    //         }
                    //     })
                    //     .text_style(egui::TextStyle::Body),
                    // ));
                    // if theme_btn.clicked() {
                    //     self.config.dark_mode = !self.config.dark_mode;
                    // }

                    // config button
                    let config_btn = ui.add(Button::new(
                        RichText::new("🛠").text_style(egui::TextStyle::Body),
                    ));

                    if config_btn.clicked() {
                        // TODO:
                    }

                    // about button
                    let about_btn =
                        ui.add(Button::new(RichText::new("ℹ").text_style(TextStyle::Body)));
                    
                        if about_btn.clicked() {
                        // self.toggle_about = !self.toggle_about;
                    }
                });
            });
            ui.add_space(10.);
        });
    }

    fn render_minefield(&mut self, ctx: &Context, _frame: &mut Frame) {
        CentralPanel::default().show(ctx, |ui| {
            let size = 30.0;
            TableBuilder::new(ui)
                .cell_layout(Layout::centered_and_justified(Direction::LeftToRight))
                .columns(Size::Absolute { initial: size - 2.0, range: (size - 2.0, size - 2.0) }, self.minefield.width() as usize)
                .body(|mut body| {
                    for y in 0..self.minefield.height() {
                        body.row(size + 2.0, |mut row| {
                            for x in 0..self.minefield.width() {
                                row.col(|ui| {
                                    self.add_spot_tile(x, y, size, ui);                                   
                                });
                            }
                        });
                    }
                });
        });        
        
    }

    fn add_spot_tile(&mut self, x: u16, y: u16, size: f32, ui: &mut Ui) {
        let spot = self.minefield.spot(x, y).unwrap();
        match spot.state() {
            SpotState::Hidden => { 
                let response = ui.button(
                    RichText::new(" ")
                        .monospace()
                        .size(size)
                );

                if response.clicked_by(PointerButton::Primary) {
                    if self.minefield.step(x, y) == StepResult::Boom {
                        // TODO: We stepped on a mine!
                    }                    
                } else if response.clicked_by(PointerButton::Secondary) {
                    self.minefield.flag(x, y);
                }              
            },
            SpotState::Revealed => {
                match spot.kind() {
                    SpotKind::Mine => { 
                        let response = ui
                            .label(
                            RichText::new("☢")
                                .color(Color32::RED)
                                .monospace()
                                .size(size)
                            );
                    },
                    SpotKind::Empty(n) => {
                        let empty_label;
                        
                        if *n > 0 { 
                            empty_label = 
                                Label::new(RichText::new(format!("{}", n))
                                .color(Color32::WHITE)
                                .monospace()
                                .size(size)
                            );
                        } else {
                            empty_label = 
                                Label::new(RichText::new(" ")
                                .color(Color32::WHITE)
                                .monospace()
                                .size(size)
                            );
                        }

                        let empty_label = ui.add(empty_label.sense(Sense::click()));

                        if empty_label.clicked_by(PointerButton::Middle) {
                            let step_result = self.minefield.try_resolve_step(x, y);

                            if step_result == StepResult::Boom {
                                // TODO: We stepped on a mine!
                            }

                            println!("{:?}", step_result);
                        }
                    },
                }
            },
            SpotState::Flagged => { 
                let response = ui.button(
                    RichText::new("⚐")
                        .color(Color32::GREEN)
                        .monospace()
                        .size(size)
                );

                if response.clicked_by(PointerButton::Secondary) {
                    self.minefield.flag(x, y);
                }
            },
        }
    }
}

impl App for MinesweepRsApp {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        ctx.request_repaint();
        ctx.set_debug_on_hover(false);

        self.render_top_panel(ctx, frame);

        self.render_minefield(ctx, frame);
    }
}

impl Default for MinesweepRsApp {
    fn default() -> Self {
        Self {
            minefield: Minefield::new(10, 10).with_mines(10),
            game_state: GameState::Stopped,
        }
    }
}

pub enum GameState {
    Running,
    Stopped
}
