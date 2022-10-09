use eframe::{
    egui::{PointerButton, self, Layout, Label, RichText, Button, Context, TextStyle, Ui, CentralPanel, Sense, Direction, TopBottomPanel, Window}, 
    epaint::{Color32, Vec2},
    Frame, NativeOptions, App,
};
use egui_extras::{TableBuilder, Size};
use minefield::{Minefield, SpotState, StepResult, SpotKind};
use std::sync::mpsc::{channel, Receiver};
use chrono::Duration;
use timer::{Timer, Guard};

mod minefield;

fn main() {
    let mut options = NativeOptions::default();
    let app = MinesweepRsApp::default();

    // FIXME: Solve auto resizing
    let size_x = 38.0;
    let size_y = 44.0;
    options.initial_window_size = Some(
        Vec2::new(
            size_x * app.minefield.width() as f32, 
            size_y * app.minefield.height() as f32
        )
    );
    options.resizable = false;

    eframe::run_native(
        "Minesweep-Rs",
        options,
        Box::new(|_cc| Box::new(app)),
    );
}

struct MinesweepRsApp {
    minefield: Minefield,
    placed_flags: i32,
    timer: AppTimer,
    seconds_lapsed: i32,
    game_state: GameState,
    show_settings: bool,
    show_about: bool,
}

impl App for MinesweepRsApp {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        ctx.request_repaint();
        ctx.set_debug_on_hover(false);
        
        self.render_top_panel(ctx, frame);
        self.render_bottom_panel(ctx, frame);
        self.render_about(ctx, frame);
        self.render_settings(ctx, frame);
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
    const WON_COLOR: Color32 = Color32::GREEN;
    const LOST_COLOR: Color32 = Color32::RED;
    const READY_COLOR: Color32 = Color32::GRAY;
    const FLAG_COUNT_OK_COLOR: Color32 = Color32::GRAY;
    const FLAG_COUNT_ERR_COLOR: Color32 = Color32::LIGHT_RED;

    fn render_top_panel(&mut self, ctx: &Context, _: &mut Frame) {
        // Service app timer
        while self.timer.poll().is_some() {
            self.seconds_lapsed += 1;
        }
                
        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.add_space(10.);
            egui::menu::bar(ui, |ui| {
                
                // Config and game data
                ui.with_layout(Layout::left_to_right(egui::Align::TOP), |ui| {
                    // refresh btn
                    let refresh_btn = ui.add(
                        Button::new(
                            RichText::new("🔄").text_style(TextStyle::Heading),
                        )
                    );
                    
                    if refresh_btn.clicked() {
                        let mines = self.minefield.mines();
                        let width = self.minefield.width();
                        let height = self.minefield.height();

                        self.minefield = Minefield::new(width, height).with_mines(mines);
                        self.placed_flags = 0;
                        self.seconds_lapsed = 0;
                        self.timer = AppTimer::default();
                        self.game_state = GameState::Ready;
                        self.show_about = false;
                        self.show_settings = false;
                    }
                    
                    ui.separator();

                    ui.allocate_ui_with_layout(Vec2::new(10.0, 10.0), Layout::left_to_right(egui::Align::Center), |ui| {
                        ui.add(
                            Label::new(
                            RichText::new("Mines").text_style(TextStyle::Body)
                        ));
                        ui.add(
                            Label::new(
                            RichText::new(format!("{}", self.minefield.mines())).monospace().text_style(TextStyle::Heading)
                        ));
                    });

                    ui.separator();

                    ui.allocate_ui_with_layout(Vec2::new(10.0, 10.0), Layout::left_to_right(egui::Align::Center), |ui| {
                        ui.add(
                            Label::new(
                            RichText::new("Flags").text_style(TextStyle::Body)
                        ));

                        let flag_count_color = if (self.minefield.mines() as i32 - self.placed_flags) >= 0 { Self::FLAG_COUNT_OK_COLOR } else { Self::FLAG_COUNT_ERR_COLOR };
                        ui.add(
                            Label::new(
                                RichText::new(format!("{}", self.minefield.mines() as i32 - self.placed_flags))
                                .color(flag_count_color)
                                .monospace()
                                .text_style(TextStyle::Heading)
                        ));
                    });

                    ui.separator();

                    ui.allocate_ui_with_layout(Vec2::new(10.0, 10.0), Layout::left_to_right(egui::Align::Center), |ui| {
                        ui.add(
                            Label::new(
                            RichText::new("Time").text_style(TextStyle::Body)
                        ));
                        ui.add(
                            Label::new(
                            RichText::new(format!("{}", self.seconds_lapsed)).monospace().text_style(TextStyle::Heading)
                        ));
                    });

                    ui.separator();
                });

                // controls
                ui.with_layout(Layout::right_to_left(egui::Align::TOP), |ui| {

                    let settings_btn = ui.add(
                        Button::new(
                            RichText::new("🛠").text_style(TextStyle::Heading),
                        )
                    );

                    if settings_btn.clicked() {
                        self.show_settings = true;
                    }

                    // about button
                    let about_btn = ui.add(Button::new(RichText::new("ℹ").text_style(TextStyle::Heading)));
                    
                    if about_btn.clicked() {
                        self.show_about = true;
                    }
                });
            });
            ui.add_space(10.);
        });
    }

    pub fn render_settings(&mut self, ctx: &Context, _: &mut Frame) {
        let window = Window::new("Settings").open(&mut self.show_settings);
        window.show(ctx, |ui| {
            let info = Label::new("TODO!");
            ui.add(info);
        });
    }

    pub fn render_about(&mut self, ctx: &Context, _: &mut Frame) {
        let window = Window::new("About Minesweep-Rs").open(&mut self.show_about);
        window.show(ctx, |ui| {
            let info = Label::new("A Rust implementation of the popular game, using the `egui` library.");
            ui.add(info);
            ui.hyperlink("https://github.com/BogdanOlar/minesweep-rs");
        });
    }

    fn render_bottom_panel(&mut self, ctx: &Context, _: &mut Frame) {
        // define a TopBottomPanel widget
        TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                match self.game_state {
                    GameState::Ready => {
                        ui.add(Label::new(
                            RichText::new("Ready")
                                .small()
                                .color(Self::READY_COLOR)
                                .text_style(TextStyle::Monospace),
                        ));
                    },
                    GameState::Running => {                   
                    },
                    GameState::Stopped(is_won) => {
                        if is_won {
                            ui.add(Label::new(
                                RichText::new("You WON!")
                                    .color(Self::WON_COLOR)
                                    .text_style(TextStyle::Monospace),
                            ));
                        } else {
                            ui.add(Label::new(
                                RichText::new("You lost.")
                                    .color(Self::LOST_COLOR)
                                    .text_style(TextStyle::Monospace),
                            ));
                        }
                    },
                }
            })
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
        self.timer.stop();
        // TODO: show statistics

        println!("Running->Stopped (lost)");
    }

    fn check_ready_to_running(&mut self) {
        if self.game_state == GameState::Ready {
            self.game_state = GameState::Running;
            self.timer.start();
            println!("Ready->Running");
        }
    }

    fn check_running_to_stopped(&mut self) {
        if self.game_state == GameState::Running && self.minefield.is_cleared() {
            self.game_state = GameState::Stopped(true);
            self.timer.stop();
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
            seconds_lapsed: 0,
            timer: AppTimer::default(),
            game_state: GameState::Ready,
            show_settings: false,
            show_about: false,
        }
    }
}

#[derive(Default)]
pub struct AppTimer {
    timer: Option<Timer>,
    guard: Option<Guard>,
    rx: Option<Receiver<()>>
}

impl AppTimer {
    pub fn stop(&mut self) {
        self.guard = None;
        self.timer = None;
        self.rx = None;
    }

    pub fn start(&mut self) {
        let (tx, rx) = channel();
        let timer = Timer::new();
        let guard = timer.schedule_repeating(Duration::seconds(1), move || {
                tx.send(()).unwrap();
        });

        self.timer = Some(timer);
        self.guard = Some(guard);
        self.rx = Some(rx);
    }    

    pub fn poll(&self) -> Option<()> {
        if let Some(rx) = &self.rx {
            rx.try_iter().next()
        } else {
            None
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
