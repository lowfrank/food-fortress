//! The frontend module is responsible for display the GUI and handling events, 
//! with the support of the backend.

use chrono::Datelike;
use eframe::egui;

use super::backend::{play_eating_sound, today, BestBefore, Food, FoodState, Fridge};

/// Return an [`egui::Label`] and [`egui::widgets::DragValue`]
macro_rules! new_label_and_drag_value {
    ($text:expr, $value:expr, $range:expr) => {
        (
            egui::Label::new(egui::WidgetText::RichText(
                egui::RichText::new($text)
                    .strong()
                    .color(egui::Color32::LIGHT_GRAY),
            )),
            egui::widgets::DragValue::new($value)
                .clamp_range($range)
                .speed(0.05),
        )
    };
}

/// The [`App`] is responsible for drawing the ui components and handling events
#[derive(Default)]
pub struct App {
    add_food_menu: AddFoodMenu,
    table: Table,
}

impl eframe::App for App {
    /// Main update
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.add_food_menu.ui(ui);
            self.add_separator(ui);
            self.table.ui(ui);
        });
    }
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Add the custom fonts
        setup_custom_fonts(&cc.egui_ctx);
        Default::default()
    }

    /// Add a separator with some space on top and bottom
    fn add_separator(&self, ui: &mut egui::Ui) {
        ui.add_space(7.0);
        ui.separator();
        ui.add_space(7.0);
    }
}

/// The [`AddFoodMenu`] lets user insert a new food in the [`Fridge`].
pub struct AddFoodMenu {
    new_food_name: String,
    new_day: u8,
    new_month: u8,

    /// This field defines how many copies of the new [`Food`] should be inserted
    /// in the [`Fridge`].
    quantity: u8,
}

impl Default for AddFoodMenu {
    fn default() -> Self {
        let today = BestBefore::today();
        Self {
            new_food_name: String::new(),
            new_day: today.day,
            new_month: today.month,
            quantity: 1,
        }
    }
}

impl AddFoodMenu {
    const FONT_SIZE: f32 = 18.0;

    /// Render the[`AddFoodMenu`]
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.collapsing(
                egui::RichText::new("Add food")
                    .strong()
                    .heading()
                    .font(egui::FontId::new(
                        Table::HEADER_FONT_SIZE - 2.0,
                        egui::FontFamily::Proportional,
                    )),
                |ui| {
                    egui::Grid::new("add food menu grid").show(ui, |ui| {
                        self.set_default_font(ui);
                        ui.end_row();
                        ui.vertical(|ui| {
                            
                            // Food name field
                            ui.horizontal(|ui| {
                                ui.add(
                                    egui::widgets::TextEdit::singleline(&mut self.new_food_name)
                                        .text_color(egui::color::Color32::WHITE)
                                        .hint_text(egui::WidgetText::RichText(
                                            egui::RichText::new("Name")
                                                .strong()
                                                .color(egui::Color32::GRAY),
                                        )),
                                );
                            });
                        });
                        ui.end_row();

                        ui.horizontal(|ui| {
                            let enabled = self.should_add_food_to_fridge();
                            // add_enabled_sized was written by me because it wasn't included
                            // in the egui library. The function adds a widget with some size
                            // and with the flag of being enabled or not.
                            let ok_button = ui.add_enabled_sized(
                                enabled,
                                [62.0; 2],
                                egui::widgets::Button::new(
                                    egui::RichText::new("Ok")
                                        .strong()
                                        .color(if enabled {
                                            egui::Color32::WHITE
                                        } else {
                                            egui::Color32::GRAY
                                        })
                                        .size(Self::FONT_SIZE),
                                ),
                            );
                            ui.end_row();

                            if ok_button.clicked() {
                                self.capitalize_new_food_name();
                                for _ in 0..self.quantity {
                                    let food = Food::new(
                                        self.new_food_name.clone(),
                                        self.new_day,
                                        self.new_month,
                                    );
                                    // This way we reset the id and foods are unique
                                    Fridge::open().add(food).update();
                                }
                                self.reset_fields();
                            }
                            ui.add_space(2.6);

                            ui.vertical(|ui| {

                                // Day section
                                ui.horizontal(|ui| {
                                    let (label, drag_value) = new_label_and_drag_value!(
                                        "Day     ",
                                        &mut self.new_day,
                                        1_u8..=31_u8
                                    );
                                    ui.add(label);
                                    ui.add_space(4.0);
                                    ui.add(drag_value);
                                });
                                
                                // Month section
                                ui.horizontal(|ui| {
                                    let (label, drag_value) = new_label_and_drag_value!(
                                        "Month  ",
                                        &mut self.new_month,
                                        1_u8..=12_u8
                                    );
                                    ui.add(label);
                                    ui.add_space(7.0);
                                    ui.add(drag_value);
                                });
                            
                                // Quantity section
                                ui.horizontal(|ui| {
                                    let (label, drag_value) = new_label_and_drag_value!(
                                        "Quantity",
                                        &mut self.quantity,
                                        1_u8..=10_u8
                                    );
                                    ui.add(label);
                                    ui.add_space(0.3);
                                    ui.add(drag_value);
                                });
                            });
                        });
                    });
                },
            );

            // Display the today date
            ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                ui.add_space(20.0);
                let today = today();
                let (day, month) = (today.day(), today.month());
                ui.add(egui::Label::new(
                    egui::RichText::new(format!("{} / {}", day, month))
                        .strong()
                        .heading()
                        .font(egui::FontId::new(
                            Table::HEADER_FONT_SIZE - 2.0,
                            egui::FontFamily::Proportional,
                        )),
                ));
            });
        });
    }

    /// Set ui default font
    #[inline]
    fn set_default_font(&self, ui: &mut egui::Ui) {
        ui.style_mut().override_font_id = Some(egui::FontId::new(
            Self::FONT_SIZE,
            egui::FontFamily::Proportional,
        ))
    }

    /// We add the food to the fridge if
    #[inline]
    fn should_add_food_to_fridge(&self) -> bool {
        matches!(
            self.new_food_name.chars().next(),
            Some(ch) if ch.is_ascii() && BestBefore::would_be_valid(self.new_day, self.new_month)
        )
    }

    /// Capitalizes the first letter of the food name, because I like it
    #[inline]
    fn capitalize_new_food_name(&mut self) {
        self.new_food_name
            .get_mut(0..1)
            .unwrap() // Guarded by length check
            .make_ascii_uppercase();
    }

    /// Reset the fields of the
    #[inline]
    fn reset_fields(&mut self) {
        self.new_food_name.clear();
        self.quantity = 1;
    }
}

/// The [`Table`] contains the information related to the single [`Food`] items.
/// Each row is a [`Food`] element.
#[derive(Default)]
pub struct Table;

impl Table {
    const BEST_BEFORE_COLUMN_WIDTH: f32 = 200.0;
    const FOOD_EATEN_BUTTON_COLUMN_WIDTH: f32 = 137.0;
    const ROW_HEIGHT: f32 = 26.0;
    const HEADER_FONT_SIZE: f32 = 32.0;
    const HEADER_HEIGHT: f32 = 46.0;
    const FONT_SIZE: f32 = 23.0;

    /// Render [`Table`]
    pub fn ui(&self, ui: &mut egui::Ui) {
        egui_extras::StripBuilder::new(ui)
            .size(egui_extras::Size::remainder())
            .vertical(|mut strip| {
                strip.cell(|ui| {
                    egui_extras::TableBuilder::new(ui)
                        .striped(true)
                        .column(egui_extras::Size::remainder())
                        .column(
                            egui_extras::Size::initial(Self::BEST_BEFORE_COLUMN_WIDTH)
                                .at_least(Self::BEST_BEFORE_COLUMN_WIDTH)
                                .at_most(Self::BEST_BEFORE_COLUMN_WIDTH),
                        )
                        .column(
                            egui_extras::Size::initial(Self::FOOD_EATEN_BUTTON_COLUMN_WIDTH)
                                .at_least(Self::FOOD_EATEN_BUTTON_COLUMN_WIDTH)
                                .at_most(Self::FOOD_EATEN_BUTTON_COLUMN_WIDTH),
                        )
                        .header(Self::HEADER_HEIGHT, |mut header| {
                            header.col(|ui| {
                                ui.vertical_centered_justified(|ui| {
                                    ui.add(self.header_label("Food"));
                                });
                            });
                            header.col(|ui| {
                                ui.add(self.header_label("Best before"));
                            });
                            header.col(|ui| {
                                ui.add(egui::Label::new(String::new()));
                            });
                        })
                        .body(|mut body| {
                            let fridge = Fridge::open();
                            for mut food in fridge {
                                body.row(Self::ROW_HEIGHT, |mut row| {
                                    row.col(|ui| {
                                        ui.add(self.cell_label(&food.name));
                                    });
                                    row.col(|ui| {
                                        let color = egui::Color32::from(food.best_before);
                                        ui.vertical_centered_justified(|ui| {
                                            ui.add(self.cell_label_with_color(
                                                food.best_before.to_string(),
                                                color,
                                            ));
                                        });
                                    });
                                    row.col(|ui| {
                                        let button_text = if food.open { "Open" } else { "Eaten" };

                                        //  Eaten food button, has fixed size
                                        if ui
                                            .add_sized(
                                                (
                                                    Self::FOOD_EATEN_BUTTON_COLUMN_WIDTH + 2.0,
                                                    Self::ROW_HEIGHT,
                                                ),
                                                egui::widgets::Button::new(
                                                    egui::RichText::new(button_text).font(
                                                        // Avoid borrow `*self` both mut and not mut
                                                        egui::FontId::new(
                                                            Self::FONT_SIZE,
                                                            egui::FontFamily::Proportional,
                                                        ),
                                                    ),
                                                ),
                                            )
                                            .clicked()
                                        {
                                            // Remove food from fridge
                                            Fridge::open().remove(&food).update();
                                            if food.open {
                                                play_eating_sound();
                                            } else {
                                                food.open = true;

                                                // Add the food back but with the open state
                                                Fridge::open().add(food).update();
                                            }
                                        }
                                    });
                                });
                            }
                        });
                })
            });
    }

    /// New header label with given text
    #[inline]
    fn header_label(&self, text: impl Into<String>) -> egui::widgets::Label {
        egui::widgets::Label::new(
            egui::RichText::new(text)
                .strong()
                .heading()
                .underline()
                .font(egui::FontId::new(
                    Self::HEADER_FONT_SIZE,
                    egui::FontFamily::Proportional,
                )),
        )
        .wrap(false)
    }

    /// New cell label with given text
    #[inline]
    fn cell_label(&self, text: impl Into<String>) -> egui::widgets::Label {
        egui::widgets::Label::new(
            egui::RichText::new(text)
                .strong()
                .color(egui::Color32::WHITE)
                .font(self.default_font()),
        )
        .wrap(false)
    }

    /// New cell label with given text and color
    #[inline]
    fn cell_label_with_color(
        &self,
        text: impl Into<String>,
        color: egui::color::Color32,
    ) -> egui::widgets::Label {
        egui::widgets::Label::new(
            egui::RichText::new(text)
                .strong()
                .color(color)
                .font(self.default_font()),
        )
        .wrap(false)
    }

    /// Return the default font
    #[inline]
    fn default_font(&self) -> egui::FontId {
        egui::FontId::new(Self::FONT_SIZE, egui::FontFamily::Proportional)
    }
}

/// Translate the [`BestBefore`] into a [`egui::Color32`]
impl From<BestBefore> for egui::Color32 {
    fn from(best_before: BestBefore) -> Self {
        match best_before.state() {
            FoodState::FarFromExpiring => Self::GREEN,
            FoodState::CloseFromExpiring => Self::YELLOW,
            FoodState::Expired => Self::RED,
        }
    }
}

/// Add custom fonts to the UI
#[inline]
fn setup_custom_fonts(ctx: &egui::Context) {
    // Start with the default fonts (we will be adding to them rather than replacing them).
    let mut fonts = egui::FontDefinitions::default();

    // Install my own font
    fonts.font_data.insert(
        "my_font".to_owned(),
        egui::FontData::from_static(include_bytes!("../../fonts/ClassicRobot-gemR.ttf")),
    );

    // Put my font first (highest priority) for proportional text:
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "my_font".to_owned());

    // Tell egui to use these fonts
    ctx.set_fonts(fonts);
}
