//! The backend mod helps the gui to run smoothly, providing multiple structs for
//! serializing and deserializing the json fridge, adding and removing foods from it
//! and updating it, as well as other helper functions such as [`play_eating_sound`]

use chrono::Datelike;
use eframe::egui;
use serde_derive::{Deserialize, Serialize};
use std::cmp;
use std::fmt;
use std::fs;
use std::io;
use std::thread;

use super::log;

pub type Foods = Vec<Food>;

/// Path to the sound the app emits when a [`Food`] has been completely eaten
const EATING_SOUND: &str = "sounds\\minecraft_eating_sound.mp3";

/// Path to the json file containing the fridge raw data
const JSON: &str = "json\\fridge.json";

// We know Feb doesn't always have 29 days but we don't care
const DAY_COUNT_FOR_MONTH: [u8; 12] = [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
const MONTHS: [u8; 12] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];

/// A [`Food`] can have one of three states
pub enum FoodState {
    FarFromExpiring,   // Green
    CloseFromExpiring, // Yellow
    Expired,           // Red
}

/// TODO: implement the year field
#[derive(Deserialize, Serialize, Eq, PartialEq, Copy, Clone)]
pub struct BestBefore {
    pub day: u8,
    pub month: u8,
}

/// Compare [`BestBefore`] in order to sort them in the UI
impl PartialOrd for BestBefore {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        match self.month.cmp(&other.month) {
            // If the two structs have the same month, then compare the day
            cmp::Ordering::Equal => Some(self.day.cmp(&other.day)),
            order => Some(order),
        }
    }
}

/// Compare [`BestBefore`] in order to sort them in the UI
impl Ord for BestBefore {
    #[inline]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        match self.month.cmp(&other.month) {
            // If the two structs have the same month, then compare the day
            cmp::Ordering::Equal => self.day.cmp(&other.day),
            order => order,
        }
    }
}

impl fmt::Display for BestBefore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:0>2} / {:0>2}", self.day, self.month) // Two digits padding
    }
}

impl BestBefore {
    #[inline]
    pub fn new(day: u8, month: u8) -> Self {
        Self { day, month }
    }

    /// Get the [`BestBefore`] of a [`Food`] of today
    #[inline]
    pub fn today() -> Self {
        let today = chrono::offset::Local::now();
        Self {
            day: today.day() as u8,
            month: today.month() as u8,
        }
    }

    /// Returns whether the given day and month would be valid in a calendar
    #[inline]
    pub fn would_be_valid(day: u8, month: u8) -> bool {
        MONTHS.contains(&month) && (1..=DAY_COUNT_FOR_MONTH[month as usize - 1]).contains(&day)
    }

    /// Based on the days left, return a [`FoodState`].
    /// The current implementation is (days are inclusive):
    ///   * Today => Expired
    ///   * Tomorrow, 2, 3  => Close from expiring
    ///   * 4 or more => Far from expiring
    #[inline]
    pub fn state(&self) -> FoodState {
        let days_left = self.days_left();
        match days_left {
            0 => FoodState::Expired,
            1..=3 => FoodState::CloseFromExpiring,
            _ => FoodState::FarFromExpiring,
        }
    }

    /// Return how many days passed since the beginning of the year until self
    #[inline]
    fn days_count(&self) -> u16 {
        let days_from_month = (0..self.month - 1)
            .into_iter()
            .fold(0, |count, i| count + DAY_COUNT_FOR_MONTH[i as usize] as u16);
        days_from_month + self.day as u16
    }

    /// Get the difference between:
    ///   * days passed since the beginning of the year until today
    ///   * days passed since the beginning of the year until self
    ///
    /// If self has more days than today, then we are in a good spot.
    /// If today has more days than self, then the food must be expired for sure
    #[inline]
    fn days_left(&self) -> u16 {
        let today = BestBefore::today();
        let days_left = self.days_count();
        let days_right = today.days_count();
        days_left.checked_sub(days_right).unwrap_or_default()
    }
}

impl From<BestBefore> for egui::WidgetText {
    fn from(date: BestBefore) -> Self {
        Self::RichText(egui::RichText::new(date.to_string()))
    }
}

/// The [`Food`] represents a single element of the [`Fridge`].
#[derive(Deserialize, Serialize, Eq, PartialEq, Clone)]
pub struct Food {
    pub name: String,
    pub best_before: BestBefore,

    /// The id field is used when sorting [`Food`]s
    pub id: u64,

    /// true when the [`Food`] has been opened but not completely eaten
    pub open: bool,
}

/// Compare [`Food`] in order to sort them in the UI
impl PartialOrd for Food {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        match self.best_before.partial_cmp(&other.best_before) {
            Some(cmp::Ordering::Equal) => match self.name.partial_cmp(&other.name) {
                Some(cmp::Ordering::Equal) => self.id.partial_cmp(&other.id),
                other => other,
            },
            other => other,
        }
    }
}

/// Compare [`Food`] in order to sort them in the UI
impl Ord for Food {
    #[inline]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        match self.best_before.cmp(&other.best_before) {
            cmp::Ordering::Equal => match self.name.cmp(&other.name) {
                cmp::Ordering::Equal => self.id.cmp(&other.id),
                other => other,
            },
            other => other,
        }
    }
}

impl Food {
    #[inline]
    pub fn new(name: String, day: u8, month: u8) -> Self {
        // Grab the foods, get the max id. If the fridge is empty, return 0 as the max id.
        // Eventually, add 1 to it.
        let id = Fridge::open()
            .foods
            .into_iter()
            .map(|food| food.id)
            .max()
            .unwrap_or(0)
            + 1;
        let best_before = BestBefore::new(day, month);
        Self {
            name,
            best_before,
            id,
            open: false,
        }
    }
}

/// A [`Fridge`] is just a collection of [`Food`]s.
#[derive(Deserialize, Serialize)]
pub struct Fridge {
    foods: Foods,
}

impl IntoIterator for Fridge {
    type Item = Food;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.foods.into_iter()
    }
}

impl Fridge {
    /// Open the [`Fridge`]. If we get an error in either reading the json or deserializing,
    /// simply [`panic`] and log the error
    #[inline]
    pub fn open() -> Self {
        let file = fs::OpenOptions::new()
            .read(true)
            .open(JSON)
            .unwrap_or_else(|err| log::error(err));
        serde_json::from_reader(file).unwrap_or_else(|err| log::error(err))
    }

    /// Update the [`Fridge`], overwriting the contents of the json file
    #[inline]
    pub fn update(&mut self) {
        self.foods.sort();
        let contents = serde_json::to_string_pretty(self).unwrap_or_else(|err| log::error(err));
        fs::write(JSON, contents).unwrap_or_else(|err| log::error(err));
    }

    /// Add a [`Food`] to the [`Fridge`]
    #[inline]
    pub fn add(mut self, food: Food) -> Self {
        self.foods.push(food);
        self
    }

    /// Remove a food from the fridge
    #[inline]
    pub fn remove(self, food: &Food) -> Self {
        let foods = self
            .foods
            .into_iter()
            .filter(|f| f != food)
            .collect::<Foods>();
        Self { foods }
    }
}

/// Play the eating sound whenever a food has been completely eaten.
/// If we can't play the sound for whatever reason, just log the error and skip the sound
pub fn play_eating_sound() {
    // https://stackoverflow.com/questions/69393226/different-behavior-between-match-and-unwrap
    // DO NOT REPLACE '_stream' WITH '_'
    thread::spawn(|| {
        let (_stream, handle) = match rodio::OutputStream::try_default() {
            Ok((s, h)) => (s, h),
            Err(err) => {
                log::warning(format!("Sound cannot be played due to an error that occurred while getting the default output device: {}", err));
                return;
            }
        };

        let sink = match rodio::Sink::try_new(&handle) {
            Ok(s) => s,
            Err(err) => {
                log::warning(format!("Sound cannot be played due to an error that occurred while creating the stream playback: {}", err));
                return;
            }
        };

        let file = match fs::File::open(EATING_SOUND) {
            Ok(f) => f,
            Err(err) => {
                log::warning(format!(
                    "Sound cannot be played due to an error that occurred while trying to read the sound file '{}': {}",
                    EATING_SOUND, err
                ));
                return;
            }
        };

        let buf = io::BufReader::new(file);
        let source = match rodio::Decoder::new(buf) {
            Ok(b) => b,
            Err(err) => {
                log::warning(format!(
                    "Sound cannot be played due to an error that occurred while decoding the sound file '{}': {}",
                    EATING_SOUND, err
                ));
                return;
            }
        };

        sink.append(source);
        sink.sleep_until_end();
    });
}

/// Return a [`chrono::DateTime`] struct with fields updated at today
pub fn today() -> chrono::DateTime<chrono::Local> {
    chrono::offset::Local::now()
}
