// Numera Tokenizer
// Lexical analysis for mathematical expressions.
// Pure function — no side effects, no state.

/// Token types
#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    Number,
    Identifier,
    Operator,
    LeftParen,
    RightParen,
    Comma,
    Assign,
    Factorial,
    Percent,
}

/// Operators
#[derive(Debug, Clone, PartialEq)]
pub enum Operator {
    Add,
    Sub,
    Mul,
    Div,
    Pow,
    BitAnd,
    BitOr,
    Shl,
    Shr,
    UnaryMinus,
}

impl Operator {
    pub fn precedence(&self) -> u8 {
        match self {
            Operator::UnaryMinus => 9,
            Operator::Pow => 7,
            Operator::Div => 6,
            Operator::Mul => 5,
            Operator::Shl | Operator::Shr => 4,
            Operator::Add | Operator::Sub => 3,
            Operator::BitAnd => 2,
            Operator::BitOr => 0,
        }
    }

    pub fn is_right_associative(&self) -> bool {
        matches!(self, Operator::Pow)
    }
}

/// A single token
#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub text: String,
}

/// Tokenize an expression string into a sequence of tokens.
///
/// `radix_char` is the decimal separator ('.' or ',').
/// Tokenize a mathematical expression into a sequence of tokens.
///
/// # Examples
///
/// ```
/// use numera::tokenizer::{tokenize, TokenType};
///
/// let tokens = tokenize("2 + 3", '.').unwrap();
/// assert_eq!(tokens.len(), 3);
/// assert_eq!(tokens[0].token_type, TokenType::Number);
/// assert_eq!(tokens[1].token_type, TokenType::Operator);
/// assert_eq!(tokens[2].token_type, TokenType::Number);
/// ```
pub fn tokenize(input: &str, radix_char: char) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = input.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        let ch = chars[i];

        // Skip whitespace
        if ch.is_whitespace() {
            i += 1;
            continue;
        }

        let pos = i;

        // Number: digit, or radix_char followed by digit, or 0x/0b/0o prefix
        if ch.is_ascii_digit()
            || (ch == radix_char && i + 1 < len && chars[i + 1].is_ascii_digit())
            || (ch == '#' && i + 1 < len && chars[i + 1].is_ascii_hexdigit())
        {
            let mut num_str = String::new();

            // Check for base prefix
            if ch == '0' && i + 1 < len {
                let next = chars[i + 1];
                if next == 'x'
                    || next == 'X'
                    || next == 'b'
                    || next == 'B'
                    || next == 'o'
                    || next == 'O'
                    || next == 'd'
                    || next == 'D'
                {
                    num_str.push(ch);
                    num_str.push(next);
                    i += 2;
                    while i < len && (chars[i].is_ascii_hexdigit() || chars[i] == '_') {
                        if chars[i] != '_' {
                            num_str.push(chars[i]);
                        }
                        i += 1;
                    }
                    tokens.push(Token {
                        token_type: TokenType::Number,
                        text: num_str,
                    });
                    continue;
                }
            }

            // Hex shorthand: #FF
            if ch == '#' {
                num_str.push(ch);
                i += 1;
                while i < len && chars[i].is_ascii_hexdigit() {
                    num_str.push(chars[i]);
                    i += 1;
                }
                tokens.push(Token {
                    token_type: TokenType::Number,
                    text: num_str,
                });
                continue;
            }

            // Regular decimal number
            while i < len && (chars[i].is_ascii_digit() || chars[i] == '_') {
                if chars[i] != '_' {
                    num_str.push(chars[i]);
                }
                i += 1;
            }

            // Radix character (decimal point)
            if i < len && chars[i] == radix_char {
                num_str.push('.');
                i += 1;
                while i < len && (chars[i].is_ascii_digit() || chars[i] == '_') {
                    if chars[i] != '_' {
                        num_str.push(chars[i]);
                    }
                    i += 1;
                }
            }

            // Exponent
            if i < len && (chars[i] == 'e' || chars[i] == 'E') {
                num_str.push('e');
                i += 1;
                if i < len && (chars[i] == '+' || chars[i] == '-') {
                    num_str.push(chars[i]);
                    i += 1;
                }
                while i < len && chars[i].is_ascii_digit() {
                    num_str.push(chars[i]);
                    i += 1;
                }
            }

            tokens.push(Token {
                token_type: TokenType::Number,
                text: num_str,
            });
            continue;
        }

        // Identifier (variable or function name)
        if ch.is_ascii_alphabetic() || ch == '_' || ch == '$' {
            let mut ident = String::new();
            while i < len
                && (chars[i].is_ascii_alphanumeric() || chars[i] == '_' || chars[i] == '$')
            {
                ident.push(chars[i]);
                i += 1;
            }
            tokens.push(Token {
                token_type: TokenType::Identifier,
                text: ident,
            });
            continue;
        }

        // Operators and punctuation
        match ch {
            '+' => {
                tokens.push(Token {
                    token_type: TokenType::Operator,
                    text: "+".to_string(),
                });
                i += 1;
            }
            '-' => {
                tokens.push(Token {
                    token_type: TokenType::Operator,
                    text: "-".to_string(),
                });
                i += 1;
            }
            '*' => {
                if i + 1 < len && chars[i + 1] == '*' {
                    tokens.push(Token {
                        token_type: TokenType::Operator,
                        text: "**".to_string(),
                    });
                    i += 2;
                } else {
                    tokens.push(Token {
                        token_type: TokenType::Operator,
                        text: "*".to_string(),
                    });
                    i += 1;
                }
            }
            '/' => {
                tokens.push(Token {
                    token_type: TokenType::Operator,
                    text: "/".to_string(),
                });
                i += 1;
            }
            '^' => {
                tokens.push(Token {
                    token_type: TokenType::Operator,
                    text: "^".to_string(),
                });
                i += 1;
            }
            '%' => {
                tokens.push(Token {
                    token_type: TokenType::Percent,
                    text: "%".to_string(),
                });
                i += 1;
            }
            '!' => {
                tokens.push(Token {
                    token_type: TokenType::Factorial,
                    text: "!".to_string(),
                });
                i += 1;
            }
            '&' => {
                tokens.push(Token {
                    token_type: TokenType::Operator,
                    text: "&".to_string(),
                });
                i += 1;
            }
            '|' => {
                tokens.push(Token {
                    token_type: TokenType::Operator,
                    text: "|".to_string(),
                });
                i += 1;
            }
            '~' => {
                tokens.push(Token {
                    token_type: TokenType::Operator,
                    text: "~".to_string(),
                });
                i += 1;
            }
            '(' => {
                tokens.push(Token {
                    token_type: TokenType::LeftParen,
                    text: "(".to_string(),
                });
                i += 1;
            }
            ')' => {
                tokens.push(Token {
                    token_type: TokenType::RightParen,
                    text: ")".to_string(),
                });
                i += 1;
            }
            ',' | ';' => {
                tokens.push(Token {
                    token_type: TokenType::Comma,
                    text: ",".to_string(),
                });
                i += 1;
            }
            '=' => {
                tokens.push(Token {
                    token_type: TokenType::Assign,
                    text: "=".to_string(),
                });
                i += 1;
            }
            '<' => {
                if i + 1 < len && chars[i + 1] == '<' {
                    tokens.push(Token {
                        token_type: TokenType::Operator,
                        text: "<<".to_string(),
                    });
                    i += 2;
                } else {
                    return Err(format!("Unexpected character '<' at position {}", pos));
                }
            }
            '>' => {
                if i + 1 < len && chars[i + 1] == '>' {
                    tokens.push(Token {
                        token_type: TokenType::Operator,
                        text: ">>".to_string(),
                    });
                    i += 2;
                } else {
                    return Err(format!("Unexpected character '>' at position {}", pos));
                }
            }
            _ => {
                return Err(format!("Unexpected character '{}' at position {}", ch, pos));
            }
        }
    }

    Ok(tokens)
}
