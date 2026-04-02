// Numera Keypad
// On-screen calculator keypad with polished button grid.

use super::theme::Theme;

/// Keypad button types
#[derive(Debug, Clone, Copy)]
pub enum KeypadKey {
    Digit(u8),
    RadixChar,
    Plus,
    Minus,
    Times,
    Divide,
    Power,
    Percent,
    LeftParen,
    RightParen,
    Clear,
    Equals,
    Pi,
    Ans,
    EE,
    Factorial,
    Sqrt,
    Exp,
    Ln,
    Sin,
    Cos,
    Tan,
    Asin,
    Acos,
    Atan,
    X,
    XEquals,
    Backspace,
}

/// What pressing this key inserts into the editor.
pub enum KeyAction {
    Insert(&'static str),
    InsertChar(char),
    InsertDynamic(String),
    Clear,
    Backspace,
    Evaluate,
}

impl KeypadKey {
    pub fn action(self, radix_char: char) -> KeyAction {
        match self {
            Self::Digit(d) => KeyAction::InsertDynamic(d.to_string()),
            Self::RadixChar => KeyAction::InsertChar(radix_char),
            Self::Plus => KeyAction::InsertChar('+'),
            Self::Minus => KeyAction::InsertChar('-'),
            Self::Times => KeyAction::InsertChar('*'),
            Self::Divide => KeyAction::InsertChar('/'),
            Self::Power => KeyAction::InsertChar('^'),
            Self::Percent => KeyAction::InsertChar('%'),
            Self::LeftParen => KeyAction::InsertChar('('),
            Self::RightParen => KeyAction::InsertChar(')'),
            Self::Pi => KeyAction::Insert("pi"),
            Self::Ans => KeyAction::Insert("ans"),
            Self::EE => KeyAction::Insert("e"),
            Self::Factorial => KeyAction::InsertChar('!'),
            Self::Sqrt => KeyAction::Insert("sqrt("),
            Self::Exp => KeyAction::Insert("exp("),
            Self::Ln => KeyAction::Insert("ln("),
            Self::Sin => KeyAction::Insert("sin("),
            Self::Cos => KeyAction::Insert("cos("),
            Self::Tan => KeyAction::Insert("tan("),
            Self::Asin => KeyAction::Insert("asin("),
            Self::Acos => KeyAction::Insert("acos("),
            Self::Atan => KeyAction::Insert("atan("),
            Self::X => KeyAction::InsertChar('x'),
            Self::XEquals => KeyAction::Insert("x="),
            Self::Clear => KeyAction::Clear,
            Self::Backspace => KeyAction::Backspace,
            Self::Equals => KeyAction::Evaluate,
        }
    }
}

/// On-screen keypad widget
pub struct Keypad;

/// Button styles
#[derive(Clone, Copy)]
enum BtnStyle {
    Digit,
    Operator,
    Function,
    Action,
    Equals,
}

impl Keypad {
    fn styled_button(ui: &mut egui::Ui, label: &str, size: egui::Vec2, style: BtnStyle) -> bool {
        let (bg, fg, _bg_hover) = match style {
            BtnStyle::Digit => (Theme::BG_BUTTON, Theme::TEXT, Theme::BG_BUTTON_HOVER),
            BtnStyle::Operator => (
                egui::Color32::from_rgb(50, 45, 60),
                Theme::ACCENT_ORANGE,
                egui::Color32::from_rgb(65, 58, 75),
            ),
            BtnStyle::Function => (
                egui::Color32::from_rgb(36, 36, 50),
                egui::Color32::from_rgb(160, 180, 220),
                egui::Color32::from_rgb(48, 48, 65),
            ),
            BtnStyle::Action => (
                egui::Color32::from_rgb(55, 35, 35),
                egui::Color32::from_rgb(220, 140, 140),
                egui::Color32::from_rgb(70, 45, 45),
            ),
            BtnStyle::Equals => (
                egui::Color32::from_rgb(35, 70, 50),
                Theme::ACCENT,
                egui::Color32::from_rgb(45, 90, 60),
            ),
        };

        let btn = egui::Button::new(
            egui::RichText::new(label)
                .font(egui::FontId::monospace(14.0))
                .color(fg)
                .strong(),
        )
        .fill(bg)
        .corner_radius(egui::CornerRadius::same(6))
        .min_size(size);

        ui.add_sized(size, btn).clicked()
    }

    pub fn show(ui: &mut egui::Ui, radix_char: char, pressed: &mut Option<KeypadKey>) {
        let available = ui.available_width();
        let cols = 7.0;
        let spacing = 4.0;
        let btn_w = ((available - spacing * (cols - 1.0)) / cols).max(40.0);
        let btn_h = 30.0;
        let btn = egui::vec2(btn_w, btn_h);
        let func_btn = egui::vec2(btn_w, btn_h - 2.0);

        ui.spacing_mut().item_spacing = egui::vec2(spacing, spacing);

        // Row 1: Trig
        ui.horizontal(|ui| {
            if Self::styled_button(ui, "sin", func_btn, BtnStyle::Function) { *pressed = Some(KeypadKey::Sin); }
            if Self::styled_button(ui, "cos", func_btn, BtnStyle::Function) { *pressed = Some(KeypadKey::Cos); }
            if Self::styled_button(ui, "tan", func_btn, BtnStyle::Function) { *pressed = Some(KeypadKey::Tan); }
            if Self::styled_button(ui, "asin", func_btn, BtnStyle::Function) { *pressed = Some(KeypadKey::Asin); }
            if Self::styled_button(ui, "acos", func_btn, BtnStyle::Function) { *pressed = Some(KeypadKey::Acos); }
            if Self::styled_button(ui, "atan", func_btn, BtnStyle::Function) { *pressed = Some(KeypadKey::Atan); }
        });

        // Row 2: Functions
        ui.horizontal(|ui| {
            if Self::styled_button(ui, "√", func_btn, BtnStyle::Function) { *pressed = Some(KeypadKey::Sqrt); }
            if Self::styled_button(ui, "exp", func_btn, BtnStyle::Function) { *pressed = Some(KeypadKey::Exp); }
            if Self::styled_button(ui, "ln", func_btn, BtnStyle::Function) { *pressed = Some(KeypadKey::Ln); }
            if Self::styled_button(ui, "π", func_btn, BtnStyle::Function) { *pressed = Some(KeypadKey::Pi); }
            if Self::styled_button(ui, "x!", func_btn, BtnStyle::Function) { *pressed = Some(KeypadKey::Factorial); }
            if Self::styled_button(ui, "ans", func_btn, BtnStyle::Function) { *pressed = Some(KeypadKey::Ans); }
        });

        // Row 3: 7 8 9 ÷ x^ %
        ui.horizontal(|ui| {
            if Self::styled_button(ui, "7", btn, BtnStyle::Digit) { *pressed = Some(KeypadKey::Digit(7)); }
            if Self::styled_button(ui, "8", btn, BtnStyle::Digit) { *pressed = Some(KeypadKey::Digit(8)); }
            if Self::styled_button(ui, "9", btn, BtnStyle::Digit) { *pressed = Some(KeypadKey::Digit(9)); }
            if Self::styled_button(ui, "÷", btn, BtnStyle::Operator) { *pressed = Some(KeypadKey::Divide); }
            if Self::styled_button(ui, "xʸ", btn, BtnStyle::Operator) { *pressed = Some(KeypadKey::Power); }
            if Self::styled_button(ui, "%", btn, BtnStyle::Operator) { *pressed = Some(KeypadKey::Percent); }
        });

        // Row 4: 4 5 6 × ( )
        ui.horizontal(|ui| {
            if Self::styled_button(ui, "4", btn, BtnStyle::Digit) { *pressed = Some(KeypadKey::Digit(4)); }
            if Self::styled_button(ui, "5", btn, BtnStyle::Digit) { *pressed = Some(KeypadKey::Digit(5)); }
            if Self::styled_button(ui, "6", btn, BtnStyle::Digit) { *pressed = Some(KeypadKey::Digit(6)); }
            if Self::styled_button(ui, "×", btn, BtnStyle::Operator) { *pressed = Some(KeypadKey::Times); }
            if Self::styled_button(ui, "(", btn, BtnStyle::Operator) { *pressed = Some(KeypadKey::LeftParen); }
            if Self::styled_button(ui, ")", btn, BtnStyle::Operator) { *pressed = Some(KeypadKey::RightParen); }
        });

        // Row 5: 1 2 3 − x x=
        ui.horizontal(|ui| {
            if Self::styled_button(ui, "1", btn, BtnStyle::Digit) { *pressed = Some(KeypadKey::Digit(1)); }
            if Self::styled_button(ui, "2", btn, BtnStyle::Digit) { *pressed = Some(KeypadKey::Digit(2)); }
            if Self::styled_button(ui, "3", btn, BtnStyle::Digit) { *pressed = Some(KeypadKey::Digit(3)); }
            if Self::styled_button(ui, "−", btn, BtnStyle::Operator) { *pressed = Some(KeypadKey::Minus); }
            if Self::styled_button(ui, "x", btn, BtnStyle::Function) { *pressed = Some(KeypadKey::X); }
            if Self::styled_button(ui, "x=", btn, BtnStyle::Function) { *pressed = Some(KeypadKey::XEquals); }
        });

        // Row 6: 0 . ⌫ C + EE =
        ui.horizontal(|ui| {
            if Self::styled_button(ui, "0", btn, BtnStyle::Digit) { *pressed = Some(KeypadKey::Digit(0)); }
            if Self::styled_button(ui, &radix_char.to_string(), btn, BtnStyle::Digit) { *pressed = Some(KeypadKey::RadixChar); }
            if Self::styled_button(ui, "⌫", btn, BtnStyle::Action) { *pressed = Some(KeypadKey::Backspace); }
            if Self::styled_button(ui, "C", btn, BtnStyle::Action) { *pressed = Some(KeypadKey::Clear); }
            if Self::styled_button(ui, "+", btn, BtnStyle::Operator) { *pressed = Some(KeypadKey::Plus); }
            if Self::styled_button(ui, "EE", btn, BtnStyle::Function) { *pressed = Some(KeypadKey::EE); }
            if Self::styled_button(ui, "=", btn, BtnStyle::Equals) { *pressed = Some(KeypadKey::Equals); }
        });
    }
}
