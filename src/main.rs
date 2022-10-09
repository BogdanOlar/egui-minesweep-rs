use eframe::{
    egui::{PointerButton, self, Layout, Label, RichText, Button, Context, TextStyle, Ui, CentralPanel, Sense, Direction, TopBottomPanel}, 
    epaint::{Color32},
    Frame, NativeOptions, App,
};
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

impl App for MinesweepRsApp {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        ctx.request_repaint();
        ctx.set_debug_on_hover(false);

        self.render_top_panel(ctx, frame);

        self.render_minefield(ctx, frame);
    }
}

impl MinesweepRsApp {
    const MINE_CAHR: &str = "☢";
    const MINE_COLOR: Color32 = Color32::RED;
    const MINE_EXPLODED_CHAR: &str = "💥";
    const MINE_EPLODED_COLOR: Color32 = Color32::RED;
    const FLAG_CHAR: &str = "⚐";
    const FLAG_COLOR_CORRECT: Color32 = Color32::GREEN;
    const FLAG_COLOR_WRONG: Color32 = Color32::RED;
    const EMPTY_SPOT_CHARS: [&str; 9] = [" ", "1", "2", "3", "4", "5", "6", "7", "8"];
    const EMPTY_SPOT_COLORS: [Color32; Self::EMPTY_SPOT_CHARS.len()] = [
        Color32::WHITE, Color32::WHITE, Color32::WHITE, 
        Color32::WHITE, Color32::WHITE, Color32::WHITE, 
        Color32::WHITE, Color32::WHITE, Color32::WHITE
    ];
    const HIDDEN_SPOT_CHAR: &str = " ";
    const HIDDEN_SPOT_COLOR: Color32 = Color32::GRAY;

    fn render_top_panel(&mut self, ctx: &Context, _: &mut Frame) {
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
                        self.game_state = GameState::Ready;
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

    fn render_minefield(&mut self, ctx: &Context, _: &mut Frame) {
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
                                    self.render_spot(x, y, size, ui);                                   
                                });
                            }
                        });
                    }
                }
            );
        });       
    }

    /// Render one spot/tile at the given field coordinates
    fn render_spot(&mut self, x: u16, y: u16, size: f32, ui: &mut Ui) {
        let spot = self.minefield.spot(x, y).unwrap();

        match self.game_state {
            GameState::Ready | GameState::Running => {
                match spot.state() {
                    SpotState::Hidden => {
                        let hidden_btn = Button::new(
                            RichText::new(Self::HIDDEN_SPOT_CHAR)
                            .color(Self::HIDDEN_SPOT_COLOR)
                            .monospace()
                            .size(size)                    
                        );
                        let hidden_btn = ui.add_enabled(true, hidden_btn);

                        if hidden_btn.clicked_by(PointerButton::Primary) {
                            self.check_ready_to_running();
                            
                            if self.minefield.step(x, y) == StepResult::Boom {
                                self.game_over();
                            } else {
                                self.check_running_to_stopped();
                            }
                        }

                        if hidden_btn.clicked_by(PointerButton::Secondary) {
                            self.check_ready_to_running();
                            self.placed_flags += self.minefield.toggle_flag(x, y);
                            self.check_running_to_stopped();
                        }
                    },
                    SpotState::Revealed => {
                        if let SpotKind::Empty(n) = spot.kind() {
                            let empty_lbl = Label::new(
                                RichText::new(Self::EMPTY_SPOT_CHARS[n as usize])
                                .color(Self::EMPTY_SPOT_COLORS[n as usize])
                                .monospace()
                                .size(size)
                            );

                            let empty_lbl = ui.add_enabled(true, empty_lbl.sense(Sense::click()));

                            if empty_lbl.clicked_by(PointerButton::Middle) {
                                self.check_ready_to_running();
                                
                                if self.minefield.try_resolve_step(x, y) == StepResult::Boom {
                                    self.game_over();
                                } else {
                                    self.check_running_to_stopped();
                                }
                            }
                        } else {
                            unreachable!()
                        }
                    },
                    SpotState::Flagged => {
                        let flag_btn = Button::new(
                            RichText::new(Self::FLAG_CHAR)
                            .color(Self::FLAG_COLOR_CORRECT)
                            .monospace()
                            .size(size)
                        );
                        let flag_btn = ui.add_enabled(true, flag_btn);

                        if flag_btn.clicked_by(PointerButton::Secondary) {
                            self.placed_flags += self.minefield.toggle_flag(x, y);
                            self.check_running_to_stopped();
                        }                        
                    },
                    SpotState::Exploded => {
                        // Can't have exploded mine while gamestate is not `Stopped`
                        unreachable!()
                    },
                }
            },
            
            GameState::Stopped(is_won) => {
                match spot.state() {
                    SpotState::Hidden => {
                        match spot.kind() {
                            SpotKind::Mine => {
                                let mine_btn = Button::new(
                                    RichText::new(Self::MINE_CAHR)
                                    .color(Self::MINE_COLOR)
                                    .monospace()
                                    .size(size)
                                );
                                let _ = ui.add_enabled(false, mine_btn);
                            },
                            SpotKind::Empty(_) => {
                                let hidden_btn = Button::new(
                                    RichText::new(Self::HIDDEN_SPOT_CHAR)
                                    .color(Self::HIDDEN_SPOT_COLOR)
                                    .monospace()
                                    .size(size)                    
                                );
                                let _ = ui.add_enabled(false, hidden_btn);                                 
                            },
                        }
                    },
                    SpotState::Revealed => {
                        match spot.kind() {
                            SpotKind::Mine => {
                                unreachable!()
                            },
                            SpotKind::Empty(n) => {
                                let empty_lbl = Label::new(
                                    RichText::new(Self::EMPTY_SPOT_CHARS[n as usize])
                                    .color(Self::EMPTY_SPOT_COLORS[n as usize])
                                    .monospace()
                                    .size(size)
                                );
                                let _ = ui.add_enabled(is_won, empty_lbl);                                   
                            },
                        }
                    },
                    SpotState::Flagged => {
                        match spot.kind() {
                            SpotKind::Mine => {
                                let flag_btn = Button::new(
                                    RichText::new(Self::FLAG_CHAR)
                                    .color(Self::FLAG_COLOR_CORRECT)
                                    .monospace()
                                    .size(size)
                                );
                                let _ = ui.add_enabled(false, flag_btn);                                
                            },
                            SpotKind::Empty(_) => {
                                let flag_btn = Button::new(
                                    RichText::new(Self::FLAG_CHAR)
                                    .color(Self::FLAG_COLOR_WRONG)
                                    .monospace()
                                    .size(size)
                                );
                                let _ = ui.add_enabled(false, flag_btn);                                
                            },
                        }
                    },
                    SpotState::Exploded => {
                        match spot.kind() {
                            SpotKind::Mine => {
                                let mine_btn = Button::new(
                                    RichText::new(Self::MINE_EXPLODED_CHAR)
                                    .color(Self::MINE_EPLODED_COLOR)
                                    .monospace()
                                    .size(size)
                                );
                                let _ = ui.add_enabled(false, mine_btn);                                
                            },
                            SpotKind::Empty(_) => {
                                unreachable!()
                            },
                        }
                    },
                }           
            },
        }
    }

    fn game_over(&mut self) {
        self.game_state = GameState::Stopped(false);
        // TODO: show statistics

        println!("Running->Stopped (lost)");
    }

    fn check_ready_to_running(&mut self) {
        if self.game_state == GameState::Ready {
            self.game_state = GameState::Running;
            // TODO: start timer

            println!("Ready->Running");
        }
    }

    fn check_running_to_stopped(&mut self) {
        if self.game_state == GameState::Running && self.minefield.is_cleared() {
            self.game_state = GameState::Stopped(true);
            // TODO: show victory
            println!("Running->Stopped (won)");
        }
    }
}

impl Default for MinesweepRsApp {
    fn default() -> Self {
        Self {
            minefield: Minefield::new(10, 10).with_mines(10),
            placed_flags: 0,
            game_state: GameState::Ready,
        }
    }
}

/// Current state of the game
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum GameState {
    /// Game is ready to start running
    Ready,
    
    /// Game is running
    Running,

    /// Game is stopped, and was either won (`true`), or lost (`false`)
    Stopped(bool)
}
