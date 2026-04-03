// Numera Settings
// Application configuration with persistence via JSON file.

use crate::math::{AngleMode, NumberFormat};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Font family selection
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum FontFamily {
    Monospace,
    Proportional,
}

impl FontFamily {
    pub fn label(self) -> &'static str {
        match self {
            FontFamily::Monospace => "Monospace",
            FontFamily::Proportional => "Proportional",
        }
    }
}

/// Font settings for a UI element
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct FontSettings {
    pub family: FontFamily,
    pub size: f32,
}

/// Default font sizes
pub const DEFAULT_EXPRESSION_FONT: FontSettings = FontSettings { family: FontFamily::Monospace, size: 12.0 };
pub const DEFAULT_RESULT_FONT: FontSettings = FontSettings { family: FontFamily::Monospace, size: 15.0 };
pub const DEFAULT_INPUT_FONT: FontSettings = FontSettings { family: FontFamily::Monospace, size: 16.0 };

/// Application settings (persisted to disk)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    // Math settings
    pub angle_mode: AngleMode,
    pub result_format: NumberFormat,
    pub precision: i32,   // -1 = auto
    pub radix_char: char, // '.' or ','

    // Behavior
    pub auto_calc: bool,
    pub save_session: bool,
    pub save_variables: bool,

    // Panel visibility
    pub show_keypad: bool,
    pub show_functions: bool,
    pub show_variables: bool,
    pub show_constants: bool,
    pub show_status_bar: bool,

    // Font settings
    pub expression_font: FontSettings,
    pub result_font: FontSettings,
    pub input_font: FontSettings,

    // History
    pub history_expressions: Vec<String>,
    pub history_results: Vec<String>,

    // Variables
    pub variables: HashMap<String, String>,

    // Window geometry
    pub window_width: f32,
    pub window_height: f32,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            angle_mode: AngleMode::Radian,
            result_format: NumberFormat::General,
            precision: -1,
            radix_char: '.',

            auto_calc: true,
            save_session: true,
            save_variables: true,

            show_keypad: false,
            show_functions: false,
            show_variables: false,
            show_constants: false,
            show_status_bar: true,

            expression_font: DEFAULT_EXPRESSION_FONT,
            result_font: DEFAULT_RESULT_FONT,
            input_font: DEFAULT_INPUT_FONT,

            history_expressions: Vec::new(),
            history_results: Vec::new(),

            variables: HashMap::new(),

            window_width: 900.0,
            window_height: 600.0,
        }
    }
}

impl Settings {
    /// Get the settings file path
    fn config_path() -> PathBuf {
        let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("numera");
        fs::create_dir_all(&path).ok();
        path.push("settings.json");
        path
    }

    /// Load settings from disk
    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            match fs::read_to_string(&path) {
                Ok(data) => match serde_json::from_str(&data) {
                    Ok(settings) => return settings,
                    Err(e) => eprintln!("Failed to parse settings: {}", e),
                },
                Err(e) => eprintln!("Failed to read settings: {}", e),
            }
        }
        Settings::default()
    }

    /// Save settings to disk
    pub fn save(&self) {
        let path = Self::config_path();
        match serde_json::to_string_pretty(self) {
            Ok(data) => {
                if let Err(e) = fs::write(&path, data) {
                    eprintln!("Failed to write settings: {}", e);
                }
            }
            Err(e) => eprintln!("Failed to serialize settings: {}", e),
        }
    }

    /// Get precision label for display
    pub fn precision_label(&self) -> String {
        match self.precision {
            -1 => "Auto".to_string(),
            p => format!("{} digits", p),
        }
    }

    /// Get angle mode label
    pub fn angle_mode_label(&self) -> &str {
        match self.angle_mode {
            AngleMode::Radian => "RAD",
            AngleMode::Degree => "DEG",
        }
    }

    /// Get format label
    pub fn format_label(&self) -> &str {
        match self.result_format {
            NumberFormat::General => "General",
            NumberFormat::Fixed => "Fixed",
            NumberFormat::Scientific => "Scientific",
            NumberFormat::Engineering => "Engineering",
            NumberFormat::Hexadecimal => "Hexadecimal",
            NumberFormat::Octal => "Octal",
            NumberFormat::Binary => "Binary",
        }
    }

    /// Reset font settings to defaults
    pub fn reset_fonts(&mut self) {
        self.expression_font = DEFAULT_EXPRESSION_FONT;
        self.result_font = DEFAULT_RESULT_FONT;
        self.input_font = DEFAULT_INPUT_FONT;
    }
}
