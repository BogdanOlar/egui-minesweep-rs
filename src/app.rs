use minefield_rs::{Minefield, SpotState, StepResult, FlagToggleResult};

use eframe::{
    egui::{PointerButton, self, Layout, Label, RichText, Button, Context, TextStyle, Ui, CentralPanel, Sense, Direction, TopBottomPanel, Window, ComboBox},
    epaint::{Color32, Vec2},
    emath::{Align},
    Frame, App, CreationContext,
};
use egui_extras::{TableBuilder, Size};
use serde::{Serialize, Deserialize};
use std::sync::mpsc::{channel, Receiver};

// Native timer
#[cfg(not(target_arch = "wasm32"))]
use timer::{Timer, Guard};

// WASM timer
#[cfg(target_arch = "wasm32")]
use gloo_timers::callback::Interval;

pub struct MinesweepRsApp {
    minefield: Minefield,
    placed_flags: u32,
    timer: AppTimer,
    seconds_lapsed: i32,
    game_state: GameState,
    game_config: GameConfig,
    ui_toolbar_group: UiToolbarGroup,
}

impl App for MinesweepRsApp {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        ctx.request_repaint();
        ctx.set_debug_on_hover(false);

        self.render_top_panel(ctx, frame);
        self.render_bottom_panel(ctx, frame);
        self.render_toolbar_group(ctx, frame);
        self.render_minefield(ctx, frame);
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, Self::APP_NAME, &self.game_config);
    }
}

impl MinesweepRsApp {
    const APP_NAME: &str = "egui minesweep-rs";
    const REFRESH_BTN_CHAR: &str = "🔄";
    const SETTINGS_BTN_CHAR: &str = "🛠";
    const ABOUT_BTN_CHAR: &str = "ℹ";
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

    pub fn with_context(mut self, cc: &CreationContext) -> Self {
        if let Some(storage) = cc.storage {
            self.game_config = eframe::get_value(storage, Self::APP_NAME).unwrap_or_default();
            tracing::debug!("Loaded config from storage {:?}", self.game_config);
        } else {
            tracing::debug!("No storage. Using default config {:?}", self.game_config);
        }

        self.minefield = Minefield::new(self.game_config.width, self.game_config.height).with_mines(self.game_config.mines);

        self
    }
    
    #[allow(dead_code)]
    pub fn with_configs(mut self, game_config: GameConfig) -> Self {
        self.game_config = game_config;
        self.minefield = Minefield::new(self.game_config.width, self.game_config.height).with_mines(self.game_config.mines);

        self
    }

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
                            RichText::new(Self::REFRESH_BTN_CHAR).text_style(TextStyle::Heading),
                        )
                    );

                    if refresh_btn.clicked() {
                        self.refresh();
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

                        let flag_count_color = if self.minefield.mines() >= self.placed_flags { Self::FLAG_COUNT_OK_COLOR } else { Self::FLAG_COUNT_ERR_COLOR };
                        ui.add(
                            Label::new(
                                RichText::new(format!("{}", self.placed_flags))
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

                    // settings button
                    if ui.add(
                        Button::new(
                            RichText::new(Self::SETTINGS_BTN_CHAR).text_style(TextStyle::Heading),
                        )
                    ).clicked() {
                        if let UiToolbarGroup::Settings(_) = self.ui_toolbar_group {
                            self.ui_toolbar_group = UiToolbarGroup::None;
                        } else {
                            self.ui_toolbar_group = UiToolbarGroup::Settings(self.game_config);
                        }
                    }

                    // about button
                    if ui.add(
                        Button::new(
                            RichText::new(Self::ABOUT_BTN_CHAR).text_style(TextStyle::Heading)
                        )
                    ).clicked() {
                        if let UiToolbarGroup::About = self.ui_toolbar_group {
                            self.ui_toolbar_group = UiToolbarGroup::None;
                        } else {
                            self.ui_toolbar_group = UiToolbarGroup::About;
                        }
                    }
                });
            });
            ui.add_space(10.);
        });
    }

    fn render_toolbar_group(&mut self, ctx: &Context, _: &mut Frame) {
        let mut open = true;

        match self.ui_toolbar_group {
            
            // About window
            UiToolbarGroup::About => {
                Window::new("About Minesweep-Rs").open(&mut open).show(ctx, |ui| {
                    ui.add(Label::new("MIT License"));
                    ui.separator();
                    ui.add(Label::new("Copyright (c) 2022 Bogdan Olar"));
                    ui.separator();
                    ui.hyperlink("https://github.com/BogdanOlar/egui-minesweep-rs");
                    ui.separator();
                    ui.add(Label::new("Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the \"Software\"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:"));
                    ui.add(Label::new("The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software."));
                    ui.add(Label::new("THE SOFTWARE IS PROVIDED \"AS IS\", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.")); 
                });
            },

            // Settings window
            UiToolbarGroup::Settings(mut game_config) => {
                Window::new("Settings").open(&mut open).show(ctx, |ui| {
                    let currently_selected = GameDifficulty::from_config(&game_config);
                    let mut selected = currently_selected;
                    
                    ComboBox::from_label("Game difficulty")
                        .selected_text(format!("{:?}", selected))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut selected, GameDifficulty::Easy, "Easy");
                            ui.selectable_value(&mut selected, GameDifficulty::Medium, "Medium");
                            ui.selectable_value(&mut selected, GameDifficulty::Hard, "Hard");
                        }
                    );

                    if selected != currently_selected {
                        tracing::debug!("\tprev {:?} {:?}", currently_selected, game_config);

                        match selected {
                            GameDifficulty::Easy => {
                                game_config = GameDifficulty::EASY;
                            },
                            GameDifficulty::Medium => {
                                game_config = GameDifficulty::MEDIUM;
                            },
                            GameDifficulty::Hard => {
                                game_config = GameDifficulty::HARD;
                            },
                        }

                        // Save the new config into the toolbar window variant (don't apply yet!)
                        self.ui_toolbar_group = UiToolbarGroup::Settings(game_config);
                        tracing::debug!("\tnew: {:?} {:?}", selected, game_config);
                    }

                    ui.with_layout(Layout::right_to_left(Align::TOP), |ui| {
                        if ui.button("Apply").clicked_by(PointerButton::Primary) {
                            tracing::debug!("\tapply: {:?}", game_config);
                            self.game_config = game_config;
                            self.refresh();
                        }

                        if ui.button("Cancel").clicked_by(PointerButton::Primary) {
                            self.ui_toolbar_group = UiToolbarGroup::None;
                        }                        
                    });
                });                
            },

            UiToolbarGroup::None => {},
        }

        // If the user closes whatever group window was open, make sure it's not rendered anymore
        if !open {
            self.ui_toolbar_group = UiToolbarGroup::None;
        }
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
                                RichText::new("You WIN!")
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
                match spot.state {
                    SpotState::HiddenEmpty { neighboring_mines: _ } | SpotState::HiddenMine => {
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
                                self.game_over(false);
                            } else if self.minefield.is_cleared() {
                                self.game_over(true);
                            }
                        }

                        if hidden_btn.clicked_by(PointerButton::Secondary) {
                            self.check_ready_to_running();
                            
                            match self.minefield.toggle_flag(x, y) {
                                FlagToggleResult::Removed => self.placed_flags -= 1,
                                FlagToggleResult::Added => self.placed_flags += 1,
                                FlagToggleResult::None => {},
                            }
                            
                            if self.minefield.is_cleared() {
                                self.game_over(true);
                            }
                        }
                    },
                    SpotState::FlaggedEmpty { neighboring_mines: _ } | SpotState::FlaggedMine => {
                        let flag_btn = Button::new(
                            RichText::new(Self::FLAG_CHAR)
                            .color(Self::FLAG_COLOR_CORRECT)
                            .monospace()
                            .size(size)
                        );
                        let flag_btn = ui.add_enabled(true, flag_btn);

                        if flag_btn.clicked_by(PointerButton::Secondary) {
                            match self.minefield.toggle_flag(x, y) {
                                FlagToggleResult::Removed => self.placed_flags -= 1,
                                FlagToggleResult::Added => self.placed_flags += 1,
                                FlagToggleResult::None => {},
                            }

                            if self.minefield.is_cleared() {
                                self.game_over(true);
                            }
                        }
                    },

                    SpotState::RevealedEmpty { neighboring_mines } => {
                        let empty_lbl = Label::new(
                            RichText::new(Self::EMPTY_SPOT_CHARS[neighboring_mines as usize])
                            .color(Self::EMPTY_SPOT_COLORS[neighboring_mines as usize])
                            .monospace()
                            .size(size)
                        );

                        let empty_lbl = ui.add_enabled(true, empty_lbl.sense(Sense::click()));

                        if empty_lbl.clicked_by(PointerButton::Middle) {
                            self.check_ready_to_running();

                            if self.minefield.auto_step(x, y) == StepResult::Boom {
                                self.game_over(false);
                            } else if self.minefield.is_cleared() {
                                self.game_over(true);
                            }
                        }
                    },
                    SpotState::ExplodedMine => {
                        unreachable!()
                    },
                }
            },

            GameState::Stopped(is_won) => {
                match spot.state {
                    SpotState::HiddenEmpty { neighboring_mines: _ } => {
                        let _ = ui.add_enabled(false, Button::new(
                            RichText::new(Self::HIDDEN_SPOT_CHAR)
                            .color(Self::HIDDEN_SPOT_COLOR)
                            .monospace()
                            .size(size)
                        ));                        
                    },
                    SpotState::HiddenMine => {
                        let _ = ui.add_enabled(false, Button::new(
                            RichText::new(Self::MINE_CAHR)
                            .color(Self::MINE_COLOR)
                            .monospace()
                            .size(size)
                        ));
                    },
                    SpotState::FlaggedEmpty { neighboring_mines: _ } => {
                        let _ = ui.add_enabled(false, Button::new(
                            RichText::new(Self::FLAG_CHAR)
                            .color(Self::FLAG_COLOR_WRONG)
                            .monospace()
                            .size(size)
                        ));
                    },
                    SpotState::FlaggedMine => {
                        let _ = ui.add_enabled(false, Button::new(
                            RichText::new(Self::FLAG_CHAR)
                            .color(Self::FLAG_COLOR_CORRECT)
                            .monospace()
                            .size(size)
                        ));
                    },
                    SpotState::RevealedEmpty { neighboring_mines } => {
                        let _ = ui.add_enabled(is_won, Label::new(
                            RichText::new(Self::EMPTY_SPOT_CHARS[neighboring_mines as usize])
                            .color(Self::EMPTY_SPOT_COLORS[neighboring_mines as usize])
                            .monospace()
                            .size(size)
                        ));
                    },
                    SpotState::ExplodedMine => {
                        let _ = ui.add_enabled(false, Button::new(
                            RichText::new(Self::MINE_EXPLODED_CHAR)
                            .color(Self::MINE_EPLODED_COLOR)
                            .monospace()
                            .size(size)
                        ));
                    },
                }
            },
        }
    }

    fn game_over(&mut self, is_won: bool) {
        self.game_state = GameState::Stopped(is_won);
        self.timer.stop();
    }

    fn check_ready_to_running(&mut self) {
        if self.game_state == GameState::Ready {
            self.game_state = GameState::Running;
            self.timer.start();
        }
    }

    fn refresh(&mut self) {
        let minefield = Minefield::new(self.game_config.width, self.game_config.height).with_mines(self.game_config.mines);
        let game_config = self.game_config;
        *self = Self {
            minefield,
            game_config,
            ..Default::default()
        };
    }

}

impl Default for MinesweepRsApp {
    fn default() -> Self {
        let game_config = GameConfig::default();
        Self {
            minefield: Minefield::new(game_config.width, game_config.height).with_mines(game_config.mines),
            placed_flags: 0,
            seconds_lapsed: 0,
            timer: AppTimer::default(),
            game_state: GameState::default(),
            game_config,
            ui_toolbar_group: UiToolbarGroup::default(),
        }
    }
}

/// Current state of the game
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum GameState {
    /// Game is ready to start running
    Ready,

    /// Game is running
    Running,

    /// Game is stopped, and was either won (`true`), or lost (`false`)
    Stopped(bool)
}

impl Default for GameState {
    fn default() -> Self {
        Self::Ready
    }
}

enum UiToolbarGroup {
    None,
    About,
    Settings(GameConfig),
}

impl Default for UiToolbarGroup {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameConfig {
    pub width: u16,
    pub height: u16,
    pub mines: u32,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self { width: 10, height: 10, mines: 10 }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GameDifficulty {
    Easy,
    Medium,
    Hard,
}

impl GameDifficulty {
    pub const EASY: GameConfig = GameConfig { width: 10, height: 10, mines: 10 };
    pub const MEDIUM: GameConfig = GameConfig { width: 16, height: 16, mines: 40 };
    pub const HARD: GameConfig = GameConfig { width: 30, height: 16, mines: 99 };

    pub fn from_config(config: &GameConfig) -> Self {
        if *config == Self::EASY {
            Self::Easy
        } else if *config == Self::MEDIUM {
            Self::Medium
        } else if *config == Self::HARD {
            Self::Hard
        } else {
            unreachable!()
        }
    }
}

/// Native app timer
#[cfg(not(target_arch = "wasm32"))]
#[derive(Default)]
struct AppTimer {
    timer: Option<Timer>,
    guard: Option<Guard>,
    rx: Option<Receiver<()>>
}

/// WASM app timer
#[cfg(target_arch = "wasm32")]
#[derive(Default)]
struct AppTimer {
    timer: Option<Interval>,
    rx: Option<Receiver<()>>
}

impl AppTimer {
    pub fn stop(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.guard = None;
            self.timer = None;
            self.rx = None;
        }

        #[cfg(target_arch = "wasm32")]
        {
            if let Some(prev_interval) = self.timer.take() {
                prev_interval.cancel();
            }            
        }
    }

    pub fn start(&mut self) {
        let (tx, rx) = channel();

        #[cfg(not(target_arch = "wasm32"))]
        {
            use chrono::Duration;

            let timer = Timer::new();
            let guard = timer.schedule_repeating(Duration::seconds(1), move || {
                    tx.send(()).unwrap();
            });
    
            self.timer = Some(timer);
            self.guard = Some(guard);
            self.rx = Some(rx);
        }

        #[cfg(target_arch = "wasm32")]
        {
            let interval = Interval::new(1000, move || {
                tx.send(()).unwrap();
            }); 
            
            self.timer = Some(interval);
            self.rx = Some(rx);
        }        
    }

    pub fn poll(&self) -> Option<()> {
        if let Some(rx) = &self.rx {
            rx.try_iter().next()
        } else {
            None
        }
    }
}


#[cfg(target_arch = "wasm32")]
use eframe::wasm_bindgen::{self, prelude::*};

/// WASM entry point
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn main_web(canvas_id: &str) {
    use eframe::WebOptions;

    tracing_wasm::set_as_global_default();

    let options = WebOptions {
        follow_system_theme: false,
        default_theme: eframe::Theme::Dark,
        ..Default::default()
    };

    eframe::start_web(canvas_id, options, Box::new(|cc| Box::new(MinesweepRsApp::default().with_context(cc))))
        .expect("Failed to launch egui-minesweep-rs");
}