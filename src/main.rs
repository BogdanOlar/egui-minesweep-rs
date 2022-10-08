use eframe::{egui::{PointerButton, self, Layout, Label, RichText, Button, Context, TextStyle, Ui, CentralPanel, Sense, Direction, TopBottomPanel}, epaint::Color32, Frame, NativeOptions, App};
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
    placed_flags: i32,
    game_state: GameState,
}

impl MinesweepRsApp {
    const CHAR_MINE: &str = "☢";
    const CHAR_MINE_EXPLODED: &str = "💥";
    const COLOUR_MINE: Color32 = Color32::RED;
    const CHAR_FLAG: &str = "⚐";
    const COLOUR_FLAG_CORRECT: Color32 = Color32::GREEN;
    const COLOUR_FLAG_WRONG: Color32 = Color32::RED;
    const CHARS_EMPTY: [&str; 9] = [" ", "1", "2", "3", "4", "5", "6", "7", "8"];
    const CHAR_HIDDEN: &str = " ";

    fn render_top_panel(&mut self, ctx: &Context, frame: &mut Frame) {
        // define a TopBottomPanel widget
        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.add_space(10.);
            egui::menu::bar(ui, |ui| {
                // logo
                ui.with_layout(Layout::left_to_right(egui::Align::TOP), |ui| {
                    ui.add(Label::new(
                        RichText::new("📰").text_style(TextStyle::Heading),
                    ));
                });
                // controls
                ui.with_layout(Layout::right_to_left(egui::Align::TOP), |ui| {
                    // refresh btn
                    let refresh_btn = ui.add(Button::new(
                        RichText::new("🔄").text_style(TextStyle::Heading),
                    ));
                    
                    if refresh_btn.clicked() {
                        let mines = self.minefield.mines();
                        let width = self.minefield.width();
                        let height = self.minefield.height();

                        self.minefield = Minefield::new(width, height).with_mines(mines);
                        self.placed_flags = 0;
                        self.game_state = GameState::Running;
                    }

                    // config button
                    let config_btn = ui.add(Button::new(
                        RichText::new("🛠").text_style(TextStyle::Heading),
                    ));

                    if config_btn.clicked() {
                        // TODO:
                    }

                    // about button
                    let about_btn =
                        ui.add(Button::new(RichText::new("ℹ").text_style(TextStyle::Heading)));
                    
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
                .columns(Size::Absolute { initial: size - 1.0, range: (size - 1.0, size - 1.0) }, self.minefield.width() as usize)
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
                }
            );
        });        
        
    }

    fn add_spot_tile(&mut self, x: u16, y: u16, size: f32, ui: &mut Ui) {
        let spot = self.minefield.spot(x, y).unwrap();
        match spot.state() {
            SpotState::Hidden => {
                    let hidden_btn;

                    if self.game_state == GameState::Running || SpotKind::Mine != spot.kind(){
                        hidden_btn = Button::new(
                            RichText::new(Self::CHAR_HIDDEN)
                            .monospace()
                            .size(size)
                        );
                    } else {
                        hidden_btn = Button::new(
                            RichText::new(Self::CHAR_MINE)
                            .color(Self::COLOUR_MINE)
                            .monospace()
                            .size(size)
                        );
                    } 

                    let hidden_btn = ui.add_enabled(self.game_state == GameState::Running, hidden_btn);
                    
                    if hidden_btn.clicked_by(PointerButton::Primary) {
                        if self.minefield.step(x, y) == StepResult::Boom {
                            self.game_state = GameState::Stopped;
                            // TODO: Show statistics
                        }                    
                    } else if hidden_btn.clicked_by(PointerButton::Secondary) {
                        self.placed_flags += self.minefield.flag(x, y);
                    }
                
                
            },
            SpotState::Revealed => {
                match spot.kind() {
                    SpotKind::Mine => {
                        let mine_lbl = Label::new(
                            RichText::new(Self::CHAR_MINE)
                            .color(Self::COLOUR_MINE)
                            .monospace()
                            .size(size)
                        );

                        let _ = ui.add_enabled(self.game_state == GameState::Running, mine_lbl);
                    },
                    SpotKind::Empty(n) => {
                        let empty_lbl = Label::new(
                            RichText::new(Self::CHARS_EMPTY[n as usize])
                            .color(Color32::WHITE)
                            .monospace()
                            .size(size)
                        );
                        
                        let empty_lbl = ui.add_enabled(self.game_state == GameState::Running, empty_lbl.sense(Sense::click()));

                        if empty_lbl.clicked_by(PointerButton::Middle) {
                            let step_result = self.minefield.try_resolve_step(x, y);

                            if step_result == StepResult::Boom {
                                self.game_state = GameState::Stopped;
                                // TODO: Show statistics
                            }
                        }
                        
                    },
                }
            },
            SpotState::Flagged => {
                let flag_btn;
                
                if self.game_state == GameState::Running || spot.kind() == SpotKind::Mine {
                    flag_btn = Button::new(
                        RichText::new(Self::CHAR_FLAG)
                        .color(Self::COLOUR_FLAG_CORRECT)
                        .monospace()
                        .size(size)                    
                    );
                } else {
                    flag_btn = Button::new(
                        RichText::new(Self::CHAR_FLAG)
                        .color(Self::COLOUR_FLAG_WRONG)
                        .monospace()
                        .size(size)                    
                    );
                }

                let flag_btn = ui.add_enabled(self.game_state == GameState::Running, flag_btn);
                
                if flag_btn.clicked_by(PointerButton::Secondary) && self.game_state == GameState::Running {
                    self.placed_flags += self.minefield.flag(x, y);
                }
                
            },
            SpotState::Exploded => {
                let mine_lbl = Label::new(
                    RichText::new(Self::CHAR_MINE_EXPLODED)
                    .color(Self::COLOUR_MINE)
                    .monospace()
                    .size(size)
                );

                let _ = ui.add_enabled(self.game_state == GameState::Running, mine_lbl);
            }
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
            placed_flags: 0,
            game_state: GameState::Stopped,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum GameState {
    Running,
    Stopped
}
