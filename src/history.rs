// Numera History
// Value object for computation history entries with format-on-demand display.

use crate::math::{HNumber, NumberFormat};

/// A single history entry storing the expression and its computed value.
///
/// The `result` field holds the formatted display string at the time of
/// evaluation, while `value` keeps the raw HNumber for reformatting.
#[derive(Clone)]
pub struct HistoryEntry {
    pub expression: String,
    pub result: String,
    pub value: HNumber,
    pub format_override: Option<NumberFormat>,
    pub is_error: bool,
}

impl HistoryEntry {
    pub fn success(
        expression: String,
        value: HNumber,
        result: String,
        format_override: Option<NumberFormat>,
    ) -> Self {
        HistoryEntry {
            expression,
            result,
            value,
            format_override,
            is_error: false,
        }
    }

    pub fn error(expression: String, message: String) -> Self {
        HistoryEntry {
            expression,
            result: message,
            value: HNumber::nan(),
            format_override: None,
            is_error: true,
        }
    }

    pub fn format_result(
        &self,
        default_format: NumberFormat,
        precision: i32,
        radix_char: char,
    ) -> String {
        if self.is_error {
            return self.result.clone();
        }
        let format = self.format_override.unwrap_or(default_format);
        self.value.format_with(format, precision, radix_char)
    }
}
