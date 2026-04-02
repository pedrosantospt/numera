// Numera Expression Evaluator
// Parser (shunting-yard) and evaluator for mathematical expressions.
// Supports variables, functions, multiple number bases, and operator precedence.

use crate::math::{HNumber, HMath, AngleMode, NumberFormat};
use crate::tokenizer::{tokenize, Token, TokenType, Operator};
use crate::functions;
use std::collections::HashMap;

/// Items on the output/operator stack during shunting-yard
#[derive(Debug, Clone)]
enum StackItem {
    Op(Operator),
    FunctionCall(String, usize), // (name, arg_count)
    LeftParen,
}

/// The evaluator context: parses and evaluates mathematical expressions.
///
/// # Examples
///
/// ```
/// use numera::evaluator::Evaluator;
/// use numera::math::NumberFormat;
///
/// let mut eval = Evaluator::new();
///
/// // Basic arithmetic
/// let (r, _) = eval.evaluate("2^10", '.').unwrap();
/// assert_eq!(r.format_with(NumberFormat::General, 15, '.'), "1024");
///
/// // Variables
/// let _ = eval.evaluate("x = 42", '.').unwrap();
/// let (r, _) = eval.evaluate("x * 2", '.').unwrap();
/// assert_eq!(r.format_with(NumberFormat::General, 15, '.'), "84");
/// ```
pub struct Evaluator {
    pub variables: HashMap<String, HNumber>,
    pub angle_mode: AngleMode,
}

impl Default for Evaluator {
    fn default() -> Self {
        Self::new()
    }
}

impl Evaluator {
    pub fn new() -> Self {
        let mut vars = HashMap::new();
        vars.insert("pi".to_string(), HMath::pi());
        vars.insert("e".to_string(), HMath::e());
        vars.insert("phi".to_string(), HMath::phi());
        vars.insert("i".to_string(), HNumber::imaginary_unit());
        vars.insert("j".to_string(), HNumber::imaginary_unit());
        vars.insert("ans".to_string(), HNumber::from_f64(0.0));
        Evaluator {
            variables: vars,
            angle_mode: AngleMode::Radian,
        }
    }

    /// Check if a name is a protected variable
    fn is_protected(name: &str) -> bool {
        matches!(name.to_lowercase().as_str(), "pi" | "e" | "phi" | "i" | "j" | "ans")
    }

    /// Set a variable (returns error if protected)
    pub fn set_variable(&mut self, name: &str, value: HNumber) -> Result<(), String> {
        let lower = name.to_lowercase();
        if matches!(lower.as_str(), "pi" | "e" | "phi" | "i" | "j") {
            return Err(format!("Cannot assign to constant '{}'", name));
        }
        self.variables.insert(lower, value);
        Ok(())
    }

    /// Get all user variables (excluding protected ones)
    pub fn user_variables(&self) -> Vec<(String, HNumber)> {
        self.variables
            .iter()
            .filter(|(k, _)| !Self::is_protected(k) && k.as_str() != "ans")
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }

    /// Delete a user variable
    #[allow(dead_code)]
    pub fn delete_variable(&mut self, name: &str) -> bool {
        let lower = name.to_lowercase();
        if Self::is_protected(&lower) {
            return false;
        }
        self.variables.remove(&lower).is_some()
    }

    /// Auto-fix expression before evaluation
    fn autofix(expr: &str, functions: &[crate::functions::FunctionDef]) -> String {
        // Strip non-printable characters (below space)
        let mut s: String = expr.chars().filter(|&c| c >= ' ').collect();
        s = s.trim().to_string();

        // Remove trailing '='
        if s.ends_with('=') && s.matches('=').count() == 1 {
            // It's an assignment, keep it
        } else {
            s = s.trim_end_matches('=').to_string();
        }

        // Auto-close parentheses
        let open = s.chars().filter(|&c| c == '(').count();
        let close = s.chars().filter(|&c| c == ')').count();
        if open > close {
            for _ in 0..(open - close) {
                s.push(')');
            }
        }

        // Special treatment: standalone function name → function(ans)
        // e.g. typing "cos" and pressing Enter evaluates cos(ans)
        let trimmed = s.trim();
        if !trimmed.is_empty() && !trimmed.contains('(') && !trimmed.contains(' ') {
            let lower = trimmed.to_lowercase();
            if functions.iter().any(|f| f.name == lower) {
                s = format!("{}(ans)", trimmed);
            }
        }

        s
    }

    /// Evaluate an expression string
    pub fn evaluate(&mut self, input: &str, radix_char: char) -> Result<(HNumber, Option<NumberFormat>), String> {
        let input = input.trim();
        if input.is_empty() {
            return Ok((HNumber::from_f64(0.0), None));
        }

        let all_funcs = functions::all_functions();
        let fixed = Self::autofix(input, &all_funcs);

        // Check for assignment: identifier = expression
        if let Some(eq_pos) = fixed.find('=') {
            let lhs = fixed[..eq_pos].trim();
            let rhs = fixed[eq_pos + 1..].trim();

            // Check if LHS is a valid identifier (not an operator expression)
            if !lhs.is_empty()
                && !rhs.is_empty()
                && lhs.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$')
                && lhs.chars().next().is_some_and(|c| c.is_ascii_alphabetic() || c == '_' || c == '$')
            {
                let lower = lhs.to_lowercase();
                if matches!(lower.as_str(), "pi" | "e" | "phi" | "i" | "j") {
                    return Err(format!("Cannot assign to constant '{}'", lhs));
                }
                // Variable can't have the same name as a function
                if all_funcs.iter().any(|f| f.name == lower) {
                    return Err(format!("'{}' matches an existing function name", lhs));
                }

                let (value, fmt) = self.eval_expression(rhs, radix_char)?;
                self.variables.insert(lower.clone(), value.clone());
                return Ok((value, fmt));
            }
        }

        let (result, fmt) = self.eval_expression(&fixed, radix_char)?;

        Ok((result, fmt))
    }

    /// Core expression evaluator using shunting-yard algorithm
    fn eval_expression(&self, input: &str, radix_char: char) -> Result<(HNumber, Option<NumberFormat>), String> {
        let tokens = tokenize(input, radix_char)?;

        if tokens.is_empty() {
            return Ok((HNumber::from_f64(0.0), None));
        }

        let mut output: Vec<(HNumber, Option<NumberFormat>)> = Vec::new();
        let mut op_stack: Vec<StackItem> = Vec::new();

        let mut i = 0;
        let len = tokens.len();

        // Track whether we expect a value or operator (for unary detection)
        let mut expect_value = true;

        while i < len {
            let token = &tokens[i];

            match &token.token_type {
                TokenType::Number => {
                    let num = HNumber::from_str_radix(&token.text)
                        .map_err(|_| format!("Invalid number: {}", token.text))?;
                    output.push((num, None));
                    expect_value = false;
                    i += 1;
                }

                TokenType::Identifier => {
                    let name_lower = token.text.to_lowercase();

                    // Check if it's a function call (followed by '(')
                    if i + 1 < len && tokens[i + 1].token_type == TokenType::LeftParen {
                        op_stack.push(StackItem::FunctionCall(name_lower, 1));
                        i += 1; // skip identifier
                        // The next token is '(' which we handle next iteration
                    } else if functions::all_functions().iter().any(|f| f.name == name_lower) {
                        // Implicit function syntax: "sin pi", "cos 1.2", "sin -90"
                        // Push as function call with 1 arg, the next value will be the arg
                        let func_name = name_lower.clone();
                        op_stack.push(StackItem::FunctionCall(func_name, 1));
                        op_stack.push(StackItem::LeftParen); // virtual paren
                        i += 1;
                        // Collect the single argument: read until we hit an operator
                        // with lower precedence, or end of tokens
                        // We use a nested mini-eval: just push a marker and let the
                        // regular parser handle it, closing at the right time
                        // Actually, simpler: read one value (possibly with unary minus)
                        let mut arg_tokens: Vec<Token> = Vec::new();
                        // Handle optional unary +/-
                        if i < len && tokens[i].token_type == TokenType::Operator
                            && (tokens[i].text == "-" || tokens[i].text == "+")
                        {
                            arg_tokens.push(tokens[i].clone());
                            i += 1;
                        }
                        // Read the value (number, identifier, or paren expression)
                        if i < len {
                            match tokens[i].token_type {
                                TokenType::Number => {
                                    arg_tokens.push(tokens[i].clone());
                                    i += 1;
                                }
                                TokenType::Identifier => {
                                    arg_tokens.push(tokens[i].clone());
                                    i += 1;
                                }
                                TokenType::LeftParen => {
                                    // Collect until matching close paren
                                    let mut depth = 0;
                                    while i < len {
                                        if tokens[i].token_type == TokenType::LeftParen { depth += 1; }
                                        if tokens[i].token_type == TokenType::RightParen { depth -= 1; }
                                        arg_tokens.push(tokens[i].clone());
                                        i += 1;
                                        if depth == 0 { break; }
                                    }
                                }
                                _ => {}
                            }
                            // Handle postfix ! or % on the arg
                            while i < len && (tokens[i].token_type == TokenType::Factorial
                                || tokens[i].token_type == TokenType::Percent)
                            {
                                arg_tokens.push(tokens[i].clone());
                                i += 1;
                            }
                        }
                        // Now evaluate the argument sub-expression
                        if arg_tokens.is_empty() {
                            // No argument - treat as function(ans)
                            if let Some(val) = self.variables.get("ans") {
                                let result = functions::call_function(&name_lower, std::slice::from_ref(val), self.angle_mode)?;
                                // Pop the virtual paren and function from stack
                                op_stack.pop(); // LeftParen
                                op_stack.pop(); // FunctionCall
                                output.push(result);
                            } else {
                                op_stack.pop();
                                op_stack.pop();
                                return Err(format!("No argument for function '{}'", token.text));
                            }
                        } else {
                            // Build a sub-expression string and evaluate it
                            let sub_expr: String = arg_tokens.iter().map(|t| t.text.as_str()).collect::<Vec<_>>().join("");
                            let (arg_val, _) = self.eval_expression(&sub_expr, radix_char)?;
                            // Pop the virtual paren and function from stack
                            op_stack.pop(); // LeftParen
                            let func_item = op_stack.pop();
                            if let Some(StackItem::FunctionCall(fname, _)) = func_item {
                                let result = functions::call_function(&fname, &[arg_val], self.angle_mode)?;
                                output.push(result);
                            }
                        }
                        expect_value = false;
                        continue; // i is already advanced
                    } else {
                        // It's a variable
                        if let Some(val) = self.variables.get(&name_lower) {
                            output.push((val.clone(), None));
                        } else {
                            return Err(format!("Unknown identifier: {}", token.text));
                        }
                        expect_value = false;
                        i += 1;
                    }
                }

                TokenType::Operator => {
                    let op = match token.text.as_str() {
                        "+" if expect_value => { i += 1; continue; } // unary plus: ignore
                        "-" if expect_value => Operator::UnaryMinus,
                        "+" => Operator::Add,
                        "-" => Operator::Sub,
                        "*" => Operator::Mul,
                        "/" => Operator::Div,
                        "^" | "**" => Operator::Pow,
                        "&" => Operator::BitAnd,
                        "|" => Operator::BitOr,
                        "~" => {
                            // Bitwise NOT: unary prefix
                            // Read next value and apply NOT
                            // For simplicity, push as unary
                            op_stack.push(StackItem::Op(Operator::UnaryMinus)); // placeholder
                            i += 1;
                            // Actually handle ~ specially
                            continue;
                        }
                        "<<" => Operator::Shl,
                        ">>" => Operator::Shr,
                        _ => return Err(format!("Unknown operator: {}", token.text)),
                    };

                    // Shunting-yard: pop operators with higher precedence
                    if !matches!(op, Operator::UnaryMinus) {
                        while let Some(top) = op_stack.last() {
                            match top {
                                StackItem::Op(top_op) => {
                                    let top_prec = top_op.precedence();
                                    let cur_prec = op.precedence();
                                    if (top_prec > cur_prec)
                                        || (top_prec == cur_prec && !op.is_right_associative())
                                    {
                                        let item = op_stack.pop().unwrap();
                                        apply_operator(&item, &mut output)?;
                                    } else {
                                        break;
                                    }
                                }
                                _ => break,
                            }
                        }
                    }

                    op_stack.push(StackItem::Op(op));
                    expect_value = true;
                    i += 1;
                }

                TokenType::LeftParen => {
                    op_stack.push(StackItem::LeftParen);
                    expect_value = true;
                    i += 1;
                }

                TokenType::RightParen => {
                    // Pop until left paren
                    let mut found_paren = false;
                    while let Some(top) = op_stack.pop() {
                        match top {
                            StackItem::LeftParen => {
                                found_paren = true;
                                break;
                            }
                            _ => {
                                apply_operator(&top, &mut output)?;
                            }
                        }
                    }

                    if !found_paren {
                        return Err("Unbalanced parentheses".to_string());
                    }

                    // If top of stack is a function, apply it
                    if let Some(StackItem::FunctionCall(name, argc)) = op_stack.last().cloned() {
                        op_stack.pop();
                        apply_function(&name, argc, &mut output, self.angle_mode)?;
                    }

                    expect_value = false;
                    i += 1;
                }

                TokenType::Comma => {
                    // Pop operators until left paren (don't pop the paren)
                    while let Some(top) = op_stack.last() {
                        match top {
                            StackItem::LeftParen => break,
                            StackItem::FunctionCall(_, _) => break,
                            _ => {
                                let item = op_stack.pop().unwrap();
                                apply_operator(&item, &mut output)?;
                            }
                        }
                    }

                    // Increment arg count of current function
                    // Find the function on the stack
                    for item in op_stack.iter_mut().rev() {
                        if let StackItem::FunctionCall(_, ref mut argc) = item {
                            *argc += 1;
                            break;
                        }
                    }

                    expect_value = true;
                    i += 1;
                }

                TokenType::Factorial => {
                    // Postfix factorial
                    if let Some((val, _)) = output.pop() {
                        output.push((HMath::factorial(&val), None));
                    } else {
                        return Err("Missing operand for factorial".to_string());
                    }
                    expect_value = false;
                    i += 1;
                }

                TokenType::Percent => {
                    // Postfix percent: multiply by 0.01
                    if let Some((val, _)) = output.pop() {
                        output.push((val * HNumber::from_f64(0.01), None));
                    } else {
                        return Err("Missing operand for percent".to_string());
                    }
                    expect_value = false;
                    i += 1;
                }

                TokenType::Assign => {
                    // Should have been handled at top level
                    return Err("Unexpected '=' in expression".to_string());
                }
            }
        }

        // Pop remaining operators
        while let Some(item) = op_stack.pop() {
            match item {
                StackItem::LeftParen => {
                    return Err("Unbalanced parentheses".to_string());
                }
                _ => {
                    apply_operator(&item, &mut output)?;
                }
            }
        }

        output.pop().ok_or_else(|| "Empty expression".to_string())
    }
}

/// Apply an operator from the stack to the output stack
fn apply_operator(item: &StackItem, output: &mut Vec<(HNumber, Option<NumberFormat>)>) -> Result<(), String> {
    match item {
        StackItem::Op(op) => {
            match op {
                Operator::UnaryMinus => {
                    let (a, _) = output.pop().ok_or("Missing operand")?;
                    output.push((-a, None));
                }
                _ => {
                    let (b, _) = output.pop().ok_or("Missing operand")?;
                    let (a, _) = output.pop().ok_or("Missing operand")?;
                    let result = match op {
                        Operator::Add => a + b,
                        Operator::Sub => a - b,
                        Operator::Mul => a * b,
                        Operator::Div => a / b,
                        Operator::Pow => HMath::raise(&a, &b),
                        Operator::BitAnd => a & b,
                        Operator::BitOr => a | b,
                        Operator::Shl => a << b,
                        Operator::Shr => a >> b,
                        _ => unreachable!(),
                    };
                    output.push((result, None));
                }
            }
        }
        StackItem::FunctionCall(name, argc) => {
            apply_function(name, *argc, output, AngleMode::Radian)?;
        }
        _ => {}
    }
    Ok(())
}

/// Apply a function call from the stack
fn apply_function(
    name: &str,
    argc: usize,
    output: &mut Vec<(HNumber, Option<NumberFormat>)>,
    angle_mode: AngleMode,
) -> Result<(), String> {
    if output.len() < argc {
        return Err(format!("Not enough arguments for function '{}'", name));
    }

    let start = output.len() - argc;
    let args: Vec<HNumber> = output.drain(start..).map(|(v, _)| v).collect();

    let result = functions::call_function(name, &args, angle_mode)?;
    output.push(result);
    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::NumberFormat;

    fn audit_exact(eval: &mut Evaluator, expr: &str, expected: &str, failures: &mut Vec<String>) {
        match eval.evaluate(expr, '.') {
            Ok((result, _)) => {
                let actual = result.format_with(NumberFormat::General, -1, '.');
                println!("[audit] {} => {}", expr, actual);
                if actual != expected {
                    failures.push(format!("{} => expected {}, got {}", expr, expected, actual));
                }
            }
            Err(err) => failures.push(format!("{} => expected {}, got error {}", expr, expected, err)),
        }
    }

    fn audit_close(
        eval: &mut Evaluator,
        expr: &str,
        expected: f64,
        tolerance: f64,
        failures: &mut Vec<String>,
    ) {
        match eval.evaluate(expr, '.') {
            Ok((result, _)) => {
                let actual = result.value();
                println!(
                    "[audit] {} => {:.17} (expected {:.17})",
                    expr, actual, expected
                );
                if !actual.is_finite() || (actual - expected).abs() > tolerance {
                    failures.push(format!(
                        "{} => expected {:.17} +/- {:.3e}, got {}",
                        expr,
                        expected,
                        tolerance,
                        result.format_with(NumberFormat::Scientific, 16, '.')
                    ));
                }
            }
            Err(err) => failures.push(format!("{} => expected {:.17}, got error {}", expr, expected, err)),
        }
    }

    fn audit_predicate<F>(
        eval: &mut Evaluator,
        expr: &str,
        description: &str,
        failures: &mut Vec<String>,
        predicate: F,
    ) where
        F: FnOnce(Result<(HNumber, Option<NumberFormat>), String>) -> Result<String, String>,
    {
        let result = predicate(eval.evaluate(expr, '.'));
        match result {
            Ok(actual) => println!("[audit] {} => {}", expr, actual),
            Err(reason) => failures.push(format!("{} => expected {}, got {}", expr, description, reason)),
        }
    }

    #[test]
    fn test_basic_eval() {
        let mut eval = Evaluator::new();
        assert_eq!(eval.evaluate("2+3", '.').unwrap().0.value(), 5.0);
        assert_eq!(eval.evaluate("10-4", '.').unwrap().0.value(), 6.0);
        assert_eq!(eval.evaluate("3*4", '.').unwrap().0.value(), 12.0);
        assert_eq!(eval.evaluate("10/4", '.').unwrap().0.value(), 2.5);
    }

    #[test]
    fn test_precedence() {
        let mut eval = Evaluator::new();
        assert_eq!(eval.evaluate("2+3*4", '.').unwrap().0.value(), 14.0);
        assert_eq!(eval.evaluate("(2+3)*4", '.').unwrap().0.value(), 20.0);
    }

    #[test]
    fn test_power() {
        let mut eval = Evaluator::new();
        assert_eq!(eval.evaluate("2^10", '.').unwrap().0.value(), 1024.0);
        assert_eq!(eval.evaluate("2**3", '.').unwrap().0.value(), 8.0);
    }

    #[test]
    fn test_unary() {
        let mut eval = Evaluator::new();
        assert_eq!(eval.evaluate("-5", '.').unwrap().0.value(), -5.0);
        assert_eq!(eval.evaluate("-3+5", '.').unwrap().0.value(), 2.0);
    }

    #[test]
    fn test_functions() {
        let mut eval = Evaluator::new();
        assert_eq!(eval.evaluate("abs(-5)", '.').unwrap().0.value(), 5.0);
        assert_eq!(eval.evaluate("sqrt(16)", '.').unwrap().0.value(), 4.0);
        assert!(eval.evaluate("sin(0)", '.').unwrap().0.value().abs() < 1e-10);
    }

    #[test]
    fn test_variables() {
        let mut eval = Evaluator::new();
        eval.evaluate("x=5", '.').unwrap();
        assert_eq!(eval.evaluate("x*2", '.').unwrap().0.value(), 10.0);
    }

    #[test]
    fn test_factorial() {
        let mut eval = Evaluator::new();
        assert_eq!(eval.evaluate("5!", '.').unwrap().0.value(), 120.0);
    }

    #[test]
    fn test_percent() {
        let mut eval = Evaluator::new();
        assert_eq!(eval.evaluate("50%", '.').unwrap().0.value(), 0.5);
    }

    #[test]
    fn test_ans() {
        let mut eval = Evaluator::new();
        let (val, _) = eval.evaluate("42", '.').unwrap();
        let _ = eval.set_variable("ans", val);
        assert_eq!(eval.evaluate("ans", '.').unwrap().0.value(), 42.0);
    }

    #[test]
    fn test_implicit_function() {
        let mut eval = Evaluator::new();
        // sin pi should work without parens
        let result = eval.evaluate("sin pi", '.').unwrap();
        assert!(result.0.value().abs() < 1e-10);
    }

    #[test]
    fn test_implicit_function_unary() {
        let mut eval = Evaluator::new();
        // cos -0 should work (unary minus in implicit syntax)
        let result = eval.evaluate("abs -5", '.').unwrap();
        assert_eq!(result.0.value(), 5.0);
    }

    #[test]
    fn test_standalone_function() {
        let mut eval = Evaluator::new();
        let (val, _) = eval.evaluate("42", '.').unwrap();
        let _ = eval.set_variable("ans", val); // ans = 42
        let result = eval.evaluate("abs", '.').unwrap(); // abs(ans) = abs(42) = 42
        assert_eq!(result.0.value(), 42.0);
    }

    #[test]
    fn test_semicolon_separator() {
        let mut eval = Evaluator::new();
        let result = eval.evaluate("ncr(5;2)", '.').unwrap();
        assert_eq!(result.0.value(), 10.0);
    }

    #[test]
    fn test_function_name_as_variable() {
        let mut eval = Evaluator::new();
        let result = eval.evaluate("sin=5", '.');
        assert!(result.is_err()); // can't assign to function name
    }

    #[test]
    fn test_sign_alias() {
        let mut eval = Evaluator::new();
        assert_eq!(eval.evaluate("sign(-5)", '.').unwrap().0.value(), -1.0);
        assert_eq!(eval.evaluate("sign(5)", '.').unwrap().0.value(), 1.0);
    }

    #[test]
    fn test_log_two_arguments() {
        let mut eval = Evaluator::new();
        let result = eval.evaluate("log(2, 2^10)", '.').unwrap();
        assert!((result.0.value() - 10.0).abs() < 1e-12);
    }

    #[test]
    fn test_validation_float_noise() {
        let mut eval = Evaluator::new();
        let result = eval.evaluate("0.1 + 0.2 - 0.3", '.').unwrap();
        assert_eq!(
            result.0.format_with(crate::math::NumberFormat::General, -1, '.'),
            "0"
        );
    }

    #[test]
    fn test_validation_euler_identity() {
        let mut eval = Evaluator::new();
        let result = eval.evaluate("e^(pi*i) + 1", '.').unwrap();
        assert_eq!(
            result.0.format_with(crate::math::NumberFormat::General, -1, '.'),
            "0"
        );
        let result_j = eval.evaluate("e^(pi*j) + 1", '.').unwrap();
        assert_eq!(
            result_j.0.format_with(crate::math::NumberFormat::General, -1, '.'),
            "0"
        );
    }

    #[test]
    fn test_validation_big_factorial_log() {
        let mut eval = Evaluator::new();
        let result = eval.evaluate("log(10000!)", '.').unwrap();
        assert!((result.0.value() - 35659.45427452078).abs() < 1e-6);
    }

    #[test]
    fn test_validation_small_delta() {
        let mut eval = Evaluator::new();
        let result = eval.evaluate("sqrt(10^16 + 1) - 10^8", '.').unwrap();
        assert!((result.0.value() - 4.9999999999999999e-9).abs() < 1e-18);
    }

    #[test]
    fn test_validation_stability() {
        let mut eval = Evaluator::new();
        let result = eval.evaluate("(1 + 1/10^15)^(10^15)", '.').unwrap();
        assert!((result.0.value() - std::f64::consts::E).abs() < 1e-12);
    }

    #[test]
    fn test_extreme_reciprocal_format() {
        let mut eval = Evaluator::new();
        let result = eval.evaluate("1 / (10^-323 / 10)", '.').unwrap();
        assert_eq!(
            result.0.format_with(NumberFormat::General, -1, '.'),
            "1e324"
        );
    }

    #[test]
    fn test_ln_large_power_of_ten_format() {
        let mut eval = Evaluator::new();
        let result = eval.evaluate("ln(10^308)", '.').unwrap();
        assert_eq!(
            result.0.format_with(NumberFormat::General, -1, '.'),
            "709.196208642166070677541368042784175941139258497662076618264993498012363780624564"
        );
    }

    #[test]
    fn test_precision_limits_audit_suite() {
        let mut eval = Evaluator::new();
        let mut failures = Vec::new();

        audit_exact(&mut eval, "0.1 + 0.2 - 0.3", "0", &mut failures);
        audit_exact(&mut eval, "1 - (3 * (1/3))", "0", &mut failures);
        audit_close(
            &mut eval,
            "sqrt(10^16 + 1) - 10^8",
            4.9999999999999999e-9,
            1e-18,
            &mut failures,
        );
        audit_close(
            &mut eval,
            "(1 + 1/10^15)^(10^15)",
            std::f64::consts::E,
            1e-12,
            &mut failures,
        );
        audit_close(
            &mut eval,
            "e^(pi * sqrt(163)) - 262537412640768744",
            -7.499274028018656e-13,
            1e-9,
            &mut failures,
        );

        audit_exact(&mut eval, "sin(pi)", "0", &mut failures);
        audit_close(
            &mut eval,
            "asin(sin(10^10))",
            -0.5092310721657347,
            1e-12,
            &mut failures,
        );
        audit_predicate(
            &mut eval,
            "tan(pi/2)",
            "error or non-finite result",
            &mut failures,
            |result| match result {
                Ok((value, _)) => {
                    let actual = value.format_with(NumberFormat::Scientific, 16, '.');
                    if !value.value().is_finite() || value.is_nan() {
                        Ok(actual)
                    } else {
                        Err(actual)
                    }
                }
                Err(err) => Ok(err),
            },
        );
        audit_close(
            &mut eval,
            "cos(10^15)",
            -0.5131937377869702,
            1e-12,
            &mut failures,
        );
        audit_close(
            &mut eval,
            "(1 - cos(10^-8)) / (10^-8)^2",
            0.5,
            1e-8,
            &mut failures,
        );

        audit_predicate(
            &mut eval,
            "1000!",
            "scientific result near 4.02387260e2567",
            &mut failures,
            |result| match result {
                Ok((value, _)) => {
                    let actual = value.format_with(NumberFormat::Scientific, 8, '.');
                    if actual.starts_with("4.0238726") && actual.ends_with("e2567") {
                        Ok(actual)
                    } else {
                        Err(actual)
                    }
                }
                Err(err) => Err(err),
            },
        );
        audit_close(&mut eval, "log(10^1000)", 1000.0, 1e-9, &mut failures);
        audit_close(&mut eval, "2^(2^(2^2))", 65536.0, 0.0, &mut failures);
        audit_predicate(
            &mut eval,
            "1 / (10^-323 / 10)",
            "finite arbitrary-precision result around 1e324",
            &mut failures,
            |result| match result {
                Ok((value, _)) => {
                    let actual = value.format_with(NumberFormat::Scientific, 6, '.');
                    if actual.ends_with("e324") {
                        Ok(actual)
                    } else {
                        Err(actual)
                    }
                }
                Err(err) => Err(err),
            },
        );
        audit_close(&mut eval, "ln(10^308)", 709.1962086421661, 1e-9, &mut failures);

        audit_close(
            &mut eval,
            "log(exp(1))",
            0.4342944819032518,
            1e-12,
            &mut failures,
        );
        audit_close(&mut eval, "125^(1/3)", 5.0, 1e-10, &mut failures);
        audit_predicate(
            &mut eval,
            "log(2, 2^10)",
            "two-argument log support returning 10",
            &mut failures,
            |result| match result {
                Ok((value, _)) => {
                    let actual = value.format_with(NumberFormat::General, -1, '.');
                    if (value.value() - 10.0).abs() < 1e-12 {
                        Ok(actual)
                    } else {
                        Err(actual)
                    }
                }
                Err(err) => Err(err),
            },
        );
        audit_predicate(
            &mut eval,
            "(-8)^(1/3)",
            "real odd root returning -2",
            &mut failures,
            |result| match result {
                Ok((value, _)) => {
                    let actual = value.format_with(NumberFormat::General, -1, '.');
                    if (value.value() + 2.0).abs() < 1e-12 {
                        Ok(actual)
                    } else {
                        Err(actual)
                    }
                }
                Err(err) => Err(err),
            },
        );
        audit_close(&mut eval, "ln(e^1000)", 1000.0, 1e-9, &mut failures);

        if !failures.is_empty() {
            panic!(
                "advanced precision audit found {} issue(s):\n{}",
                failures.len(),
                failures.join("\n")
            );
        }
    }

    // ═══════════════════════════════════════════════════════════════
    // Number format parsing
    // ═══════════════════════════════════════════════════════════════

    #[test]
    fn test_hex_literal() {
        let mut eval = Evaluator::new();
        assert_eq!(eval.evaluate("0xFF", '.').unwrap().0.value(), 255.0);
        assert_eq!(eval.evaluate("0x1A + 1", '.').unwrap().0.value(), 27.0);
    }

    #[test]
    fn test_octal_literal() {
        let mut eval = Evaluator::new();
        assert_eq!(eval.evaluate("0o10", '.').unwrap().0.value(), 8.0);
        assert_eq!(eval.evaluate("0o77", '.').unwrap().0.value(), 63.0);
    }

    #[test]
    fn test_binary_literal() {
        let mut eval = Evaluator::new();
        assert_eq!(eval.evaluate("0b1010", '.').unwrap().0.value(), 10.0);
        assert_eq!(eval.evaluate("0b11111111", '.').unwrap().0.value(), 255.0);
    }

    #[test]
    fn test_mixed_radix_arithmetic() {
        let mut eval = Evaluator::new();
        // 0xFF + 0b1 + 0o1 = 255 + 1 + 1 = 257
        assert_eq!(eval.evaluate("0xFF + 0b1 + 0o1", '.').unwrap().0.value(), 257.0);
    }

    // ═══════════════════════════════════════════════════════════════
    // Operator tests (full coverage)
    // ═══════════════════════════════════════════════════════════════

    #[test]
    fn test_power_right_associative() {
        let mut eval = Evaluator::new();
        // 2^3^2 should be 2^(3^2) = 2^9 = 512, NOT (2^3)^2 = 64
        assert_eq!(eval.evaluate("2^3^2", '.').unwrap().0.value(), 512.0);
    }

    #[test]
    fn test_double_star_power() {
        let mut eval = Evaluator::new();
        assert_eq!(eval.evaluate("3**3", '.').unwrap().0.value(), 27.0);
    }

    #[test]
    fn test_operator_bitwise_and() {
        let mut eval = Evaluator::new();
        assert_eq!(eval.evaluate("12 & 10", '.').unwrap().0.value(), 8.0);
    }

    #[test]
    fn test_operator_bitwise_or() {
        let mut eval = Evaluator::new();
        assert_eq!(eval.evaluate("12 | 10", '.').unwrap().0.value(), 14.0);
    }

    #[test]
    fn test_operator_shift_left() {
        let mut eval = Evaluator::new();
        assert_eq!(eval.evaluate("1 << 8", '.').unwrap().0.value(), 256.0);
    }

    #[test]
    fn test_operator_shift_right() {
        let mut eval = Evaluator::new();
        assert_eq!(eval.evaluate("256 >> 4", '.').unwrap().0.value(), 16.0);
    }

    #[test]
    fn test_complex_expression_i() {
        let mut eval = Evaluator::new();
        let r = eval.evaluate("2+3*i", '.').unwrap();
        let s = r.0.format_with(NumberFormat::General, -1, '.');
        assert!(s.contains('i'), "Expected complex result, got {}", s);
    }

    #[test]
    fn test_complex_j_alias() {
        let mut eval = Evaluator::new();
        let r = eval.evaluate("1+1j", '.').unwrap();
        let s = r.0.format_with(NumberFormat::General, -1, '.');
        assert!(s.contains('i') || s.contains('j'));
    }

    #[test]
    fn test_nested_parentheses() {
        let mut eval = Evaluator::new();
        assert_eq!(eval.evaluate("((2+3)*(4+1))", '.').unwrap().0.value(), 25.0);
    }

    #[test]
    fn test_unary_minus_in_expression() {
        let mut eval = Evaluator::new();
        assert_eq!(eval.evaluate("2*-3", '.').unwrap().0.value(), -6.0);
        assert_eq!(eval.evaluate("-(-5)", '.').unwrap().0.value(), 5.0);
    }

    // ═══════════════════════════════════════════════════════════════
    // Built-in constants
    // ═══════════════════════════════════════════════════════════════

    #[test]
    fn test_constant_pi() {
        let mut eval = Evaluator::new();
        assert!((eval.evaluate("pi", '.').unwrap().0.value() - std::f64::consts::PI).abs() < 1e-15);
    }

    #[test]
    fn test_constant_e() {
        let mut eval = Evaluator::new();
        assert!((eval.evaluate("e", '.').unwrap().0.value() - std::f64::consts::E).abs() < 1e-15);
    }

    #[test]
    fn test_constant_phi() {
        let mut eval = Evaluator::new();
        let phi = eval.evaluate("phi", '.').unwrap().0.value();
        assert!((phi - 1.6180339887498948).abs() < 1e-12);
    }

    #[test]
    fn test_constant_i() {
        let mut eval = Evaluator::new();
        let r = eval.evaluate("i*i", '.').unwrap();
        assert_eq!(r.0.format_with(NumberFormat::General, -1, '.'), "-1");
    }

    // ═══════════════════════════════════════════════════════════════
    // Variable assignment
    // ═══════════════════════════════════════════════════════════════

    #[test]
    fn test_variable_assignment_returns_value() {
        let mut eval = Evaluator::new();
        assert_eq!(eval.evaluate("x=42", '.').unwrap().0.value(), 42.0);
    }

    #[test]
    fn test_variable_reuse() {
        let mut eval = Evaluator::new();
        eval.evaluate("a=10", '.').unwrap();
        eval.evaluate("b=20", '.').unwrap();
        assert_eq!(eval.evaluate("a+b", '.').unwrap().0.value(), 30.0);
    }

    #[test]
    fn test_variable_overwrite() {
        let mut eval = Evaluator::new();
        eval.evaluate("x=5", '.').unwrap();
        eval.evaluate("x=10", '.').unwrap();
        assert_eq!(eval.evaluate("x", '.').unwrap().0.value(), 10.0);
    }

    #[test]
    fn test_cannot_assign_to_pi() {
        let mut eval = Evaluator::new();
        assert!(eval.evaluate("pi=42", '.').is_err());
    }

    #[test]
    fn test_cannot_assign_to_e() {
        let mut eval = Evaluator::new();
        assert!(eval.evaluate("e=42", '.').is_err());
    }

    #[test]
    fn test_case_insensitive_variable() {
        let mut eval = Evaluator::new();
        eval.evaluate("MyVar=7", '.').unwrap();
        assert_eq!(eval.evaluate("myvar", '.').unwrap().0.value(), 7.0);
    }

    // ═══════════════════════════════════════════════════════════════
    // Analysis functions (complete coverage)
    // ═══════════════════════════════════════════════════════════════

    #[test]
    fn test_fn_abs() {
        let mut eval = Evaluator::new();
        assert_eq!(eval.evaluate("abs(-42)", '.').unwrap().0.value(), 42.0);
        assert_eq!(eval.evaluate("abs(42)", '.').unwrap().0.value(), 42.0);
    }

    #[test]
    fn test_fn_average() {
        let mut eval = Evaluator::new();
        assert_eq!(eval.evaluate("average(2, 4, 6)", '.').unwrap().0.value(), 4.0);
    }

    #[test]
    fn test_fn_bin_dec_hex_oct() {
        let mut eval = Evaluator::new();
        // These return format overrides, the value stays the same
        let (v, fmt) = eval.evaluate("bin(10)", '.').unwrap();
        assert_eq!(v.value(), 10.0);
        assert_eq!(fmt, Some(NumberFormat::Binary));

        let (v, fmt) = eval.evaluate("hex(255)", '.').unwrap();
        assert_eq!(v.value(), 255.0);
        assert_eq!(fmt, Some(NumberFormat::Hexadecimal));

        let (v, fmt) = eval.evaluate("oct(8)", '.').unwrap();
        assert_eq!(v.value(), 8.0);
        assert_eq!(fmt, Some(NumberFormat::Octal));

        let (_, fmt) = eval.evaluate("dec(42)", '.').unwrap();
        assert_eq!(fmt, Some(NumberFormat::General));
    }

    #[test]
    fn test_fn_cbrt() {
        let mut eval = Evaluator::new();
        assert!((eval.evaluate("cbrt(27)", '.').unwrap().0.value() - 3.0).abs() < 1e-12);
    }

    #[test]
    fn test_fn_ceil_floor() {
        let mut eval = Evaluator::new();
        assert_eq!(eval.evaluate("ceil(3.2)", '.').unwrap().0.value(), 4.0);
        assert_eq!(eval.evaluate("ceil(-3.2)", '.').unwrap().0.value(), -3.0);
        assert_eq!(eval.evaluate("floor(3.8)", '.').unwrap().0.value(), 3.0);
        assert_eq!(eval.evaluate("floor(-3.8)", '.').unwrap().0.value(), -4.0);
    }

    #[test]
    fn test_fn_frac() {
        let mut eval = Evaluator::new();
        assert!((eval.evaluate("frac(3.75)", '.').unwrap().0.value() - 0.75).abs() < 1e-12);
    }

    #[test]
    fn test_fn_gamma() {
        let mut eval = Evaluator::new();
        // gamma(5) = 4! = 24
        assert!((eval.evaluate("gamma(5)", '.').unwrap().0.value() - 24.0).abs() < 1e-8);
    }

    #[test]
    fn test_fn_geomean() {
        let mut eval = Evaluator::new();
        // geomean(2, 8) = sqrt(16) = 4
        assert!((eval.evaluate("geomean(2, 8)", '.').unwrap().0.value() - 4.0).abs() < 1e-12);
    }

    #[test]
    fn test_fn_int() {
        let mut eval = Evaluator::new();
        assert_eq!(eval.evaluate("int(3.9)", '.').unwrap().0.value(), 3.0);
        assert_eq!(eval.evaluate("int(-3.9)", '.').unwrap().0.value(), -3.0);
    }

    #[test]
    fn test_fn_lngamma() {
        let mut eval = Evaluator::new();
        // lngamma(5) = ln(gamma(5)) = ln(24) ≈ 3.178
        assert!((eval.evaluate("lngamma(5)", '.').unwrap().0.value() - 3.178053830347946).abs() < 1e-6);
    }

    #[test]
    fn test_fn_max_min() {
        let mut eval = Evaluator::new();
        assert_eq!(eval.evaluate("max(1, 5, 3)", '.').unwrap().0.value(), 5.0);
        assert_eq!(eval.evaluate("min(1, 5, 3)", '.').unwrap().0.value(), 1.0);
    }

    #[test]
    fn test_fn_product() {
        let mut eval = Evaluator::new();
        assert_eq!(eval.evaluate("product(2, 3, 4)", '.').unwrap().0.value(), 24.0);
    }

    #[test]
    fn test_fn_round() {
        let mut eval = Evaluator::new();
        assert_eq!(eval.evaluate("round(3.7)", '.').unwrap().0.value(), 4.0);
        assert_eq!(eval.evaluate("round(3.14159, 2)", '.').unwrap().0.value(), 3.14);
    }

    #[test]
    fn test_fn_sgn() {
        let mut eval = Evaluator::new();
        assert_eq!(eval.evaluate("sgn(-42)", '.').unwrap().0.value(), -1.0);
        assert_eq!(eval.evaluate("sgn(0)", '.').unwrap().0.value(), 0.0);
        assert_eq!(eval.evaluate("sgn(42)", '.').unwrap().0.value(), 1.0);
    }

    #[test]
    fn test_fn_sqrt() {
        let mut eval = Evaluator::new();
        assert_eq!(eval.evaluate("sqrt(144)", '.').unwrap().0.value(), 12.0);
    }

    #[test]
    fn test_fn_sum() {
        let mut eval = Evaluator::new();
        assert_eq!(eval.evaluate("sum(1, 2, 3, 4)", '.').unwrap().0.value(), 10.0);
    }

    #[test]
    fn test_fn_trunc() {
        let mut eval = Evaluator::new();
        assert_eq!(eval.evaluate("trunc(3.999)", '.').unwrap().0.value(), 3.0);
        assert_eq!(eval.evaluate("trunc(3.14159, 3)", '.').unwrap().0.value(), 3.141);
    }

    // ═══════════════════════════════════════════════════════════════
    // Logarithm & Hyperbolic functions
    // ═══════════════════════════════════════════════════════════════

    #[test]
    fn test_fn_exp() {
        let mut eval = Evaluator::new();
        assert!((eval.evaluate("exp(1)", '.').unwrap().0.value() - std::f64::consts::E).abs() < 1e-12);
    }

    #[test]
    fn test_fn_ln() {
        let mut eval = Evaluator::new();
        assert!((eval.evaluate("ln(e)", '.').unwrap().0.value() - 1.0).abs() < 1e-12);
    }

    #[test]
    fn test_fn_log_base10() {
        let mut eval = Evaluator::new();
        assert!((eval.evaluate("log(1000)", '.').unwrap().0.value() - 3.0).abs() < 1e-12);
    }

    #[test]
    fn test_fn_lg() {
        let mut eval = Evaluator::new();
        // lg = log base 2
        assert!((eval.evaluate("lg(256)", '.').unwrap().0.value() - 8.0).abs() < 1e-12);
    }

    #[test]
    fn test_fn_cosh() {
        let mut eval = Evaluator::new();
        assert!((eval.evaluate("cosh(0)", '.').unwrap().0.value() - 1.0).abs() < 1e-12);
    }

    #[test]
    fn test_fn_sinh() {
        let mut eval = Evaluator::new();
        assert!(eval.evaluate("sinh(0)", '.').unwrap().0.value().abs() < 1e-12);
    }

    #[test]
    fn test_fn_tanh() {
        let mut eval = Evaluator::new();
        assert!(eval.evaluate("tanh(0)", '.').unwrap().0.value().abs() < 1e-12);
    }

    #[test]
    fn test_fn_arcosh() {
        let mut eval = Evaluator::new();
        // arcosh(1) = 0
        assert!(eval.evaluate("arcosh(1)", '.').unwrap().0.value().abs() < 1e-12);
    }

    #[test]
    fn test_fn_arsinh() {
        let mut eval = Evaluator::new();
        assert!(eval.evaluate("arsinh(0)", '.').unwrap().0.value().abs() < 1e-12);
    }

    #[test]
    fn test_fn_artanh() {
        let mut eval = Evaluator::new();
        assert!(eval.evaluate("artanh(0)", '.').unwrap().0.value().abs() < 1e-12);
    }

    // ═══════════════════════════════════════════════════════════════
    // Trigonometry functions
    // ═══════════════════════════════════════════════════════════════

    #[test]
    fn test_fn_sin_cos_tan() {
        let mut eval = Evaluator::new();
        assert!(eval.evaluate("sin(0)", '.').unwrap().0.value().abs() < 1e-15);
        assert!((eval.evaluate("cos(0)", '.').unwrap().0.value() - 1.0).abs() < 1e-15);
        assert!(eval.evaluate("tan(0)", '.').unwrap().0.value().abs() < 1e-15);
    }

    #[test]
    fn test_fn_asin_acos_atan() {
        let mut eval = Evaluator::new();
        assert!(eval.evaluate("asin(0)", '.').unwrap().0.value().abs() < 1e-15);
        // acos(1) = 0
        assert!(eval.evaluate("acos(1)", '.').unwrap().0.value().abs() < 1e-15);
        // atan(0) = 0
        assert!(eval.evaluate("atan(0)", '.').unwrap().0.value().abs() < 1e-15);
    }

    #[test]
    fn test_fn_cot_sec_csc() {
        let mut eval = Evaluator::new();
        // cot(pi/4) = 1
        assert!((eval.evaluate("cot(pi/4)", '.').unwrap().0.value() - 1.0).abs() < 1e-12);
        // sec(0) = 1/cos(0) = 1
        assert!((eval.evaluate("sec(0)", '.').unwrap().0.value() - 1.0).abs() < 1e-12);
        // csc(pi/2) = 1/sin(pi/2) = 1
        assert!((eval.evaluate("csc(pi/2)", '.').unwrap().0.value() - 1.0).abs() < 1e-12);
    }

    #[test]
    fn test_fn_degrees_radians() {
        let mut eval = Evaluator::new();
        assert!((eval.evaluate("degrees(pi)", '.').unwrap().0.value() - 180.0).abs() < 1e-10);
        assert!((eval.evaluate("radians(180)", '.').unwrap().0.value() - std::f64::consts::PI).abs() < 1e-12);
    }

    #[test]
    fn test_trig_degree_mode() {
        let mut eval = Evaluator::new();
        eval.angle_mode = crate::math::AngleMode::Degree;
        // sin(90°) = 1
        assert!((eval.evaluate("sin(90)", '.').unwrap().0.value() - 1.0).abs() < 1e-12);
        // cos(60°) = 0.5
        assert!((eval.evaluate("cos(60)", '.').unwrap().0.value() - 0.5).abs() < 1e-12);
    }

    // ═══════════════════════════════════════════════════════════════
    // Discrete functions
    // ═══════════════════════════════════════════════════════════════

    #[test]
    fn test_fn_gcd() {
        let mut eval = Evaluator::new();
        assert_eq!(eval.evaluate("gcd(12, 8)", '.').unwrap().0.value(), 4.0);
        assert_eq!(eval.evaluate("gcd(12, 8, 6)", '.').unwrap().0.value(), 2.0);
    }

    #[test]
    fn test_fn_ncr() {
        let mut eval = Evaluator::new();
        // C(10,3) = 120
        assert_eq!(eval.evaluate("ncr(10, 3)", '.').unwrap().0.value(), 120.0);
    }

    #[test]
    fn test_fn_npr() {
        let mut eval = Evaluator::new();
        // P(5,3) = 60
        assert_eq!(eval.evaluate("npr(5, 3)", '.').unwrap().0.value(), 60.0);
    }

    // ═══════════════════════════════════════════════════════════════
    // Probability functions
    // ═══════════════════════════════════════════════════════════════

    #[test]
    fn test_fn_binomial() {
        let mut eval = Evaluator::new();
        // binommean(10, 0.3) = 3
        assert!((eval.evaluate("binommean(10, 0.3)", '.').unwrap().0.value() - 3.0).abs() < 1e-10);
        // binomvar(10, 0.3) = 2.1
        assert!((eval.evaluate("binomvar(10, 0.3)", '.').unwrap().0.value() - 2.1).abs() < 1e-10);
        // binompmf: P(X=3) for n=10, p=0.3 ≈ 0.2668
        assert!((eval.evaluate("binompmf(3, 10, 0.3)", '.').unwrap().0.value() - 0.26683).abs() < 1e-3);
        // binomcdf: P(X≤3) for n=10, p=0.3 ≈ 0.6496
        assert!((eval.evaluate("binomcdf(3, 10, 0.3)", '.').unwrap().0.value() - 0.6496).abs() < 1e-3);
    }

    #[test]
    fn test_fn_erf() {
        let mut eval = Evaluator::new();
        // erf(0) = 0 (may have minor imprecision in series approximation)
        assert!(eval.evaluate("erf(0)", '.').unwrap().0.value().abs() < 0.01);
        // erfc(0) = 1
        assert!((eval.evaluate("erfc(0)", '.').unwrap().0.value() - 1.0).abs() < 0.01);
        // erf + erfc = 1 identity
        let erf1 = eval.evaluate("erf(1)", '.').unwrap().0.value();
        let erfc1 = eval.evaluate("erfc(1)", '.').unwrap().0.value();
        assert!((erf1 + erfc1 - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_fn_poisson() {
        let mut eval = Evaluator::new();
        // poissonmean(5) = 5
        assert_eq!(eval.evaluate("poissonmean(5)", '.').unwrap().0.value(), 5.0);
        // poissonvar(5) = 5
        assert_eq!(eval.evaluate("poissonvar(5)", '.').unwrap().0.value(), 5.0);
        // poissonpmf: P(X=3|λ=5) ≈ 0.1404
        assert!((eval.evaluate("poissonpmf(3, 5)", '.').unwrap().0.value() - 0.1404).abs() < 1e-3);
        // poissoncdf: P(X<=3|λ=5) — cumulative, should be > pmf and valid probability
        let cdf = eval.evaluate("poissoncdf(3, 5)", '.').unwrap().0.value();
        assert!(cdf > 0.1404 && cdf < 1.0);
    }

    #[test]
    fn test_fn_poisson_aliases() {
        let mut eval = Evaluator::new();
        // Aliases should produce same results
        let a = eval.evaluate("poissonpmf(3, 5)", '.').unwrap().0.value();
        let b = eval.evaluate("poipmf(3, 5)", '.').unwrap().0.value();
        assert!((a - b).abs() < 1e-15);
    }

    #[test]
    fn test_fn_hypergeometric() {
        let mut eval = Evaluator::new();
        // hypermean(N=50, K=10, n=5) = n*K/N = 1
        assert!((eval.evaluate("hypermean(50, 10, 5)", '.').unwrap().0.value() - 1.0).abs() < 1e-10);
        // hypervar(N=50, K=10, n=5) = n*K*(N-K)*(N-n) / (N^2*(N-1))
        // = 5*10*40*45 / (2500*49) = 90000/122500 ≈ 0.7347
        assert!((eval.evaluate("hypervar(50, 10, 5)", '.').unwrap().0.value() - 0.7347).abs() < 1e-3);
        // hyperpmf(k=2, N=50, K=10, n=5): P(X=2) for hypergeometric
        let pmf = eval.evaluate("hyperpmf(2, 50, 10, 5)", '.').unwrap().0.value();
        assert!(pmf > 0.0 && pmf < 1.0); // valid probability
        // hypercdf(k=2, N=50, K=10, n=5): P(X<=2) >= P(X=2)
        let cdf = eval.evaluate("hypercdf(2, 50, 10, 5)", '.').unwrap().0.value();
        assert!(cdf >= pmf);
        assert!(cdf > 0.0 && cdf <= 1.0);
    }

    // ═══════════════════════════════════════════════════════════════
    // Logic / Bitwise functions
    // ═══════════════════════════════════════════════════════════════

    #[test]
    fn test_fn_and_or_xor() {
        let mut eval = Evaluator::new();
        assert_eq!(eval.evaluate("and(12, 10)", '.').unwrap().0.value(), 8.0);
        assert_eq!(eval.evaluate("or(12, 10)", '.').unwrap().0.value(), 14.0);
        assert_eq!(eval.evaluate("xor(12, 10)", '.').unwrap().0.value(), 6.0);
    }

    #[test]
    fn test_fn_not() {
        let mut eval = Evaluator::new();
        // not(0) = -1 (two's complement)
        let r = eval.evaluate("not(0)", '.').unwrap().0.value();
        assert_eq!(r, -1.0);
    }

    #[test]
    fn test_fn_shl_shr() {
        let mut eval = Evaluator::new();
        assert_eq!(eval.evaluate("shl(1, 4)", '.').unwrap().0.value(), 16.0);
        assert_eq!(eval.evaluate("shr(16, 2)", '.').unwrap().0.value(), 4.0);
    }

    #[test]
    fn test_fn_idiv_mod() {
        let mut eval = Evaluator::new();
        assert_eq!(eval.evaluate("idiv(17, 5)", '.').unwrap().0.value(), 3.0);
        assert_eq!(eval.evaluate("mod(17, 5)", '.').unwrap().0.value(), 2.0);
    }

    #[test]
    fn test_fn_mask() {
        let mut eval = Evaluator::new();
        // mask(0xFF, 4) masks to 4 bits = 0xF = 15
        assert_eq!(eval.evaluate("mask(255, 4)", '.').unwrap().0.value(), 15.0);
    }

    #[test]
    fn test_fn_sgnext() {
        let mut eval = Evaluator::new();
        // sgnext(0b1000, 4) sign-extends from 4-bit: 0b1000 = -8 in 4-bit signed
        let r = eval.evaluate("sgnext(8, 4)", '.').unwrap().0.value();
        assert_eq!(r, -8.0);
    }

    // ═══════════════════════════════════════════════════════════════
    // Evaluator features
    // ═══════════════════════════════════════════════════════════════

    #[test]
    fn test_auto_close_parentheses() {
        let mut eval = Evaluator::new();
        // Missing close paren should be auto-closed
        assert_eq!(eval.evaluate("sqrt(16", '.').unwrap().0.value(), 4.0);
    }

    #[test]
    fn test_case_insensitive_functions() {
        let mut eval = Evaluator::new();
        assert_eq!(eval.evaluate("SQRT(16)", '.').unwrap().0.value(), 4.0);
        assert_eq!(eval.evaluate("Sin(0)", '.').unwrap().0.value(), 0.0);
    }

    #[test]
    fn test_case_insensitive_constants() {
        let mut eval = Evaluator::new();
        let pi1 = eval.evaluate("pi", '.').unwrap().0.value();
        let pi2 = eval.evaluate("PI", '.').unwrap().0.value();
        assert_eq!(pi1, pi2);
    }

    #[test]
    fn test_comma_as_radix_char() {
        let mut eval = Evaluator::new();
        assert_eq!(eval.evaluate("3,5 + 1,5", ',').unwrap().0.value(), 5.0);
    }

    #[test]
    fn test_semicolon_as_separator_with_comma_radix() {
        let mut eval = Evaluator::new();
        assert_eq!(eval.evaluate("max(3;7;2)", ',').unwrap().0.value(), 7.0);
    }

    #[test]
    fn test_error_unknown_identifier() {
        let mut eval = Evaluator::new();
        assert!(eval.evaluate("foobar", '.').is_err());
    }

    #[test]
    fn test_error_unbalanced_parens() {
        let mut eval = Evaluator::new();
        assert!(eval.evaluate("(2+3))", '.').is_err());
    }

    #[test]
    fn test_error_division_by_zero() {
        let mut eval = Evaluator::new();
        let r = eval.evaluate("1/0", '.');
        // Should either error or return NaN/Inf
        match r {
            Ok((val, _)) => assert!(!val.value().is_finite() || val.is_nan()),
            Err(_) => {} // also acceptable
        }
    }

    #[test]
    fn test_whitespace_only_expression() {
        let mut eval = Evaluator::new();
        // Empty or whitespace-only should either error or return 0
        let r = eval.evaluate("   ", '.');
        match r {
            Err(_) => {} // acceptable
            Ok((v, _)) => assert_eq!(v.value(), 0.0), // also acceptable
        }
    }

    #[test]
    fn test_ans_not_polluted_by_evaluate() {
        // Verify that evaluate() does NOT set ans (the bug we fixed)
        let mut eval = Evaluator::new();
        let _ = eval.set_variable("ans", HNumber::from_f64(0.0));
        eval.evaluate("42", '.').unwrap();
        // ans should still be 0 since evaluate() no longer sets it
        assert_eq!(eval.evaluate("ans", '.').unwrap().0.value(), 0.0);
    }
}
