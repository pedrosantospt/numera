// Numera Editor
// Expression input state.

pub struct Editor {
    pub text: String,
}

impl Default for Editor {
    fn default() -> Self {
        Self::new()
    }
}

impl Editor {
    pub fn new() -> Self {
        Editor {
            text: String::new(),
        }
    }
}
