// Numera Functions Library
// All built-in mathematical functions organized by category.

use crate::math::{HNumber, HMath, AngleMode, NumberFormat};

/// Function metadata
#[derive(Debug, Clone)]
pub struct FunctionDef {
    pub name: &'static str,
    #[allow(dead_code)]
    pub description: &'static str,
    #[allow(dead_code)]
    pub category: &'static str,
    pub argc_min: usize,
    pub argc_max: usize, // usize::MAX = variadic
}

/// Get all function definitions
pub fn all_functions() -> Vec<FunctionDef> {
    vec![
        // === Analysis ===
        FunctionDef { name: "abs",     description: "Absolute Value",                   category: "Analysis", argc_min: 1, argc_max: 1 },
        FunctionDef { name: "average", description: "Average (Arithmetic Mean)",        category: "Analysis", argc_min: 1, argc_max: usize::MAX },
        FunctionDef { name: "bin",     description: "Binary Representation",            category: "Analysis", argc_min: 1, argc_max: 1 },
        FunctionDef { name: "cbrt",    description: "Cube Root",                        category: "Analysis", argc_min: 1, argc_max: 1 },
        FunctionDef { name: "ceil",    description: "Ceiling",                          category: "Analysis", argc_min: 1, argc_max: 1 },
        FunctionDef { name: "dec",     description: "Decimal Representation",           category: "Analysis", argc_min: 1, argc_max: 1 },
        FunctionDef { name: "floor",   description: "Floor",                            category: "Analysis", argc_min: 1, argc_max: 1 },
        FunctionDef { name: "frac",    description: "Fractional Part",                  category: "Analysis", argc_min: 1, argc_max: 1 },
        FunctionDef { name: "gamma",   description: "Extension of Factorials [=(x-1)!]",category: "Analysis", argc_min: 1, argc_max: usize::MAX },
        FunctionDef { name: "geomean", description: "Geometric Mean",                   category: "Analysis", argc_min: 1, argc_max: usize::MAX },
        FunctionDef { name: "hex",     description: "Hexadecimal Representation",       category: "Analysis", argc_min: 1, argc_max: 1 },
        FunctionDef { name: "int",     description: "Integer Part",                     category: "Analysis", argc_min: 1, argc_max: 1 },
        FunctionDef { name: "lngamma", description: "ln(abs(Gamma))",                   category: "Analysis", argc_min: 1, argc_max: usize::MAX },
        FunctionDef { name: "max",     description: "Maximum",                          category: "Analysis", argc_min: 1, argc_max: usize::MAX },
        FunctionDef { name: "min",     description: "Minimum",                          category: "Analysis", argc_min: 1, argc_max: usize::MAX },
        FunctionDef { name: "oct",     description: "Octal Representation",             category: "Analysis", argc_min: 1, argc_max: 1 },
        FunctionDef { name: "product", description: "Product",                          category: "Analysis", argc_min: 1, argc_max: usize::MAX },
        FunctionDef { name: "round",   description: "Rounding",                         category: "Analysis", argc_min: 1, argc_max: 2 },
        FunctionDef { name: "sgn",     description: "Signum",                           category: "Analysis", argc_min: 1, argc_max: 1 },
        FunctionDef { name: "sign",    description: "Signum (alias)",                 category: "Analysis", argc_min: 1, argc_max: 1 },
        FunctionDef { name: "sqrt",    description: "Square Root",                      category: "Analysis", argc_min: 1, argc_max: 1 },
        FunctionDef { name: "sum",     description: "Sum",                              category: "Analysis", argc_min: 1, argc_max: usize::MAX },
        FunctionDef { name: "trunc",   description: "Truncation",                       category: "Analysis", argc_min: 1, argc_max: 2 },

        // === Logarithm & Hyperbolic ===
        FunctionDef { name: "arcosh",  description: "Area Hyperbolic Cosine",           category: "Logarithm & Hyperbolic", argc_min: 1, argc_max: 1 },
        FunctionDef { name: "arsinh",  description: "Area Hyperbolic Sine",             category: "Logarithm & Hyperbolic", argc_min: 1, argc_max: 1 },
        FunctionDef { name: "artanh",  description: "Area Hyperbolic Tangent",          category: "Logarithm & Hyperbolic", argc_min: 1, argc_max: 1 },
        FunctionDef { name: "cosh",    description: "Hyperbolic Cosine",                category: "Logarithm & Hyperbolic", argc_min: 1, argc_max: 1 },
        FunctionDef { name: "exp",     description: "Exponential",                      category: "Logarithm & Hyperbolic", argc_min: 1, argc_max: 1 },
        FunctionDef { name: "lg",      description: "Base-2 Logarithm",                 category: "Logarithm & Hyperbolic", argc_min: 1, argc_max: 1 },
        FunctionDef { name: "ln",      description: "Natural Logarithm",                category: "Logarithm & Hyperbolic", argc_min: 1, argc_max: 1 },
        FunctionDef { name: "log",     description: "Base-10 Logarithm or log(base, x)",category: "Logarithm & Hyperbolic", argc_min: 1, argc_max: 2 },
        FunctionDef { name: "sinh",    description: "Hyperbolic Sine",                  category: "Logarithm & Hyperbolic", argc_min: 1, argc_max: 1 },
        FunctionDef { name: "tanh",    description: "Hyperbolic Tangent",               category: "Logarithm & Hyperbolic", argc_min: 1, argc_max: 1 },

        // === Discrete ===
        FunctionDef { name: "gcd",     description: "Greatest Common Divisor",          category: "Discrete", argc_min: 2, argc_max: usize::MAX },
        FunctionDef { name: "ncr",     description: "Binomial Coefficient",             category: "Discrete", argc_min: 2, argc_max: 2 },
        FunctionDef { name: "npr",     description: "Permutation",                      category: "Discrete", argc_min: 2, argc_max: 2 },

        // === Probability ===
        FunctionDef { name: "binompmf",  description: "Binomial PMF",                   category: "Probability", argc_min: 3, argc_max: 3 },
        FunctionDef { name: "binomcdf",  description: "Binomial CDF",                   category: "Probability", argc_min: 3, argc_max: 3 },
        FunctionDef { name: "binommean", description: "Binomial Mean",                  category: "Probability", argc_min: 2, argc_max: 2 },
        FunctionDef { name: "binomvar",  description: "Binomial Variance",              category: "Probability", argc_min: 2, argc_max: 2 },
        FunctionDef { name: "erf",       description: "Error Function",                 category: "Probability", argc_min: 1, argc_max: 1 },
        FunctionDef { name: "erfc",      description: "Complementary Error Function",   category: "Probability", argc_min: 1, argc_max: 1 },
        FunctionDef { name: "hyperpmf",  description: "Hypergeometric PMF",             category: "Probability", argc_min: 4, argc_max: 4 },
        FunctionDef { name: "hypercdf",  description: "Hypergeometric CDF",             category: "Probability", argc_min: 4, argc_max: 4 },
        FunctionDef { name: "hypermean", description: "Hypergeometric Mean",            category: "Probability", argc_min: 3, argc_max: 3 },
        FunctionDef { name: "hypervar",  description: "Hypergeometric Variance",        category: "Probability", argc_min: 3, argc_max: 3 },
        FunctionDef { name: "poissonpmf",  description: "Poisson PMF",                  category: "Probability", argc_min: 2, argc_max: 2 },
        FunctionDef { name: "poissoncdf",  description: "Poisson CDF",                  category: "Probability", argc_min: 2, argc_max: 2 },
        FunctionDef { name: "poissonmean", description: "Poisson Mean",                 category: "Probability", argc_min: 1, argc_max: 1 },
        FunctionDef { name: "poissonvar",  description: "Poisson Variance",             category: "Probability", argc_min: 1, argc_max: 1 },
        FunctionDef { name: "poipmf",    description: "Poisson PMF (alias)",          category: "Probability", argc_min: 2, argc_max: 2 },
        FunctionDef { name: "poicdf",    description: "Poisson CDF (alias)",          category: "Probability", argc_min: 2, argc_max: 2 },
        FunctionDef { name: "poimean",   description: "Poisson Mean (alias)",         category: "Probability", argc_min: 1, argc_max: 1 },
        FunctionDef { name: "poivar",    description: "Poisson Variance (alias)",     category: "Probability", argc_min: 1, argc_max: 1 },

        // === Trigonometry ===
        FunctionDef { name: "acos",    description: "Arc Cosine",                       category: "Trigonometry", argc_min: 1, argc_max: 1 },
        FunctionDef { name: "asin",    description: "Arc Sine",                         category: "Trigonometry", argc_min: 1, argc_max: 1 },
        FunctionDef { name: "atan",    description: "Arc Tangent",                      category: "Trigonometry", argc_min: 1, argc_max: 1 },
        FunctionDef { name: "cos",     description: "Cosine",                           category: "Trigonometry", argc_min: 1, argc_max: 1 },
        FunctionDef { name: "cot",     description: "Cotangent",                        category: "Trigonometry", argc_min: 1, argc_max: 1 },
        FunctionDef { name: "csc",     description: "Cosecant",                         category: "Trigonometry", argc_min: 1, argc_max: 1 },
        FunctionDef { name: "degrees", description: "Radians to Degrees",               category: "Trigonometry", argc_min: 1, argc_max: 1 },
        FunctionDef { name: "radians", description: "Degrees to Radians",               category: "Trigonometry", argc_min: 1, argc_max: 1 },
        FunctionDef { name: "sec",     description: "Secant",                           category: "Trigonometry", argc_min: 1, argc_max: 1 },
        FunctionDef { name: "sin",     description: "Sine",                             category: "Trigonometry", argc_min: 1, argc_max: 1 },
        FunctionDef { name: "tan",     description: "Tangent",                          category: "Trigonometry", argc_min: 1, argc_max: 1 },

        // === Logic ===
        FunctionDef { name: "mask",    description: "Mask to a bit size",               category: "Logic", argc_min: 2, argc_max: 2 },
        FunctionDef { name: "unmask",  description: "Unmask (bitwise NOT of mask)",    category: "Logic", argc_min: 2, argc_max: 2 },
        FunctionDef { name: "sgnext",  description: "Sign-extend a value",              category: "Logic", argc_min: 2, argc_max: 2 },
        FunctionDef { name: "not",     description: "Logical NOT",                      category: "Logic", argc_min: 1, argc_max: 1 },
        FunctionDef { name: "and",     description: "Logical AND",                      category: "Logic", argc_min: 2, argc_max: usize::MAX },
        FunctionDef { name: "or",      description: "Logical OR",                       category: "Logic", argc_min: 2, argc_max: usize::MAX },
        FunctionDef { name: "xor",     description: "Logical XOR",                      category: "Logic", argc_min: 2, argc_max: usize::MAX },
        FunctionDef { name: "shl",     description: "Arithmetic Shift Left",            category: "Logic", argc_min: 2, argc_max: 2 },
        FunctionDef { name: "shr",     description: "Arithmetic Shift Right",           category: "Logic", argc_min: 2, argc_max: 2 },
        FunctionDef { name: "idiv",    description: "Integer Quotient",                 category: "Logic", argc_min: 2, argc_max: 2 },
        FunctionDef { name: "mod",     description: "Modulo",                           category: "Logic", argc_min: 2, argc_max: 2 },
    ]
}

/// Get sorted unique category names
pub fn categories() -> Vec<&'static str> {
    let mut cats: Vec<&str> = all_functions()
        .iter()
        .map(|f| f.category)
        .collect();
    cats.sort_unstable();
    cats.dedup();
    cats
}

/// Execute a function by name with given arguments.
/// Returns `(result_value, optional_format_override)`.
/// Call a built-in function by name with the given arguments.
///
/// # Examples
///
/// ```
/// use numera::functions::call_function;
/// use numera::math::{HNumber, AngleMode};
///
/// let args = vec![HNumber::from_i64(-5)];
/// let (result, _) = call_function("abs", &args, AngleMode::Radian).unwrap();
/// assert_eq!(result.format_with(numera::math::NumberFormat::General, 15, '.'), "5");
/// ```
pub fn call_function(name: &str, args: &[HNumber], angle_mode: AngleMode) -> Result<(HNumber, Option<NumberFormat>), String> {
    let name_lower = name.to_lowercase();

    // Validate arity
    let func_def = all_functions()
        .into_iter()
        .find(|f| f.name == name_lower);

    if let Some(def) = &func_def {
        if args.len() < def.argc_min {
            return Err(format!("{}() requires at least {} argument(s)", name, def.argc_min));
        }
        if args.len() > def.argc_max {
            return Err(format!("{}() takes at most {} argument(s)", name, def.argc_max));
        }
    } else {
        return Err(format!("Unknown function: {}", name));
    }

    // Helper: convert angle for trig input (deg→rad if in degree mode)
    let to_rad = |x: &HNumber| -> HNumber {
        match angle_mode {
            AngleMode::Degree => HMath::radians(x),
            AngleMode::Radian => x.clone(),
        }
    };

    // Helper: convert angle for inverse trig output (rad→deg if in degree mode)
    let from_rad = |x: HNumber| -> HNumber {
        match angle_mode {
            AngleMode::Degree => HMath::degrees(&x),
            AngleMode::Radian => x,
        }
    };

    // Format-overriding functions return the value with a format tag
    match name_lower.as_str() {
        "bin" => return Ok((args[0].clone(), Some(NumberFormat::Binary))),
        "dec" => return Ok((args[0].clone(), Some(NumberFormat::General))),
        "hex" => return Ok((args[0].clone(), Some(NumberFormat::Hexadecimal))),
        "oct" => return Ok((args[0].clone(), Some(NumberFormat::Octal))),
        _ => {}
    }

    // All other functions return value with no format override
    let result = match name_lower.as_str() {
        // === Analysis ===
        "abs" => Ok(HMath::abs(&args[0])),
        "average" => {
            let sum = args.iter().cloned().fold(HNumber::from_i64(0), |acc, value| acc + value);
            Ok(sum / HNumber::from_i64(args.len() as i64))
        }
        "cbrt" => Ok(HMath::cbrt(&args[0])),
        "ceil" => Ok(HMath::ceil(&args[0])),
        "floor" => Ok(HMath::floor(&args[0])),
        "frac" => Ok(HMath::frac(&args[0])),
        "gamma" => Ok(HMath::gamma(&args[0])),
        "geomean" => {
            let product = args.iter().cloned().fold(HNumber::from_i64(1), |acc, value| acc * value);
            Ok(HMath::raise(
                &product,
                &(HNumber::from_i64(1) / HNumber::from_i64(args.len() as i64)),
            ))
        }
        "int" => Ok(HMath::integer(&args[0])),
        "lngamma" => Ok(HMath::lngamma(&args[0])),
        "max" => {
            let mut best = args[0].clone();
            for a in &args[1..] {
                if matches!(a.partial_cmp(&best), Some(std::cmp::Ordering::Greater)) {
                    best = a.clone();
                }
            }
            Ok(best)
        }
        "min" => {
            let mut best = args[0].clone();
            for a in &args[1..] {
                if matches!(a.partial_cmp(&best), Some(std::cmp::Ordering::Less)) {
                    best = a.clone();
                }
            }
            Ok(best)
        }
        "product" => {
            Ok(args.iter().cloned().fold(HNumber::from_i64(1), |acc, value| acc * value))
        }
        "round" => {
            let decimals = if args.len() > 1 { Some(args[1].value() as i32) } else { None };
            Ok(HMath::round(&args[0], decimals))
        }
        "sgn" => Ok(HMath::sgn(&args[0])),
        "sqrt" => Ok(HMath::sqrt(&args[0])),
        "sum" => {
            Ok(args.iter().cloned().fold(HNumber::from_i64(0), |acc, value| acc + value))
        }
        "trunc" => {
            let decimals = if args.len() > 1 { Some(args[1].value() as i32) } else { None };
            Ok(HMath::trunc(&args[0], decimals))
        }

        // === Logarithm & Hyperbolic ===
        "arcosh" => Ok(HMath::arcosh(&args[0])),
        "arsinh" => Ok(HMath::arsinh(&args[0])),
        "artanh" => Ok(HMath::artanh(&args[0])),
        "cosh" => Ok(HMath::cosh(&args[0])),
        "exp" => Ok(HMath::exp(&args[0])),
        "lg" => Ok(HMath::lg(&args[0])),
        "ln" => Ok(HMath::ln(&args[0])),
        "log" => {
            if args.len() == 1 {
                Ok(HMath::log(&args[0]))
            } else {
                let ln_base = HMath::ln(&args[0]);
                let ln_value = HMath::ln(&args[1]);
                if ln_base.is_nan() || ln_value.is_nan() {
                    Ok(HNumber::nan_with_error(crate::math::MathError::OutOfDomain))
                } else if ln_base.is_zero() {
                    Ok(HNumber::nan_with_error(crate::math::MathError::DivByZero))
                } else {
                    Ok(ln_value / ln_base)
                }
            }
        }
        "sinh" => Ok(HMath::sinh(&args[0])),
        "tanh" => Ok(HMath::tanh(&args[0])),

        // === Discrete ===
        "gcd" => {
            let mut result = args[0].clone();
            for a in &args[1..] {
                result = HMath::gcd(&result, a);
            }
            Ok(result)
        }
        "ncr" => Ok(HMath::nCr(&args[0], &args[1])),
        "npr" => Ok(HMath::nPr(&args[0], &args[1])),

        // === Probability ===
        "binompmf" => Ok(HMath::binomial_pmf(&args[0], &args[1], &args[2])),
        "binomcdf" => Ok(HMath::binomial_cdf(&args[0], &args[1], &args[2])),
        "binommean" => Ok(HMath::binomial_mean(&args[0], &args[1])),
        "binomvar" => Ok(HMath::binomial_variance(&args[0], &args[1])),
        "erf" => Ok(HMath::erf(&args[0])),
        "erfc" => Ok(HMath::erfc(&args[0])),
        "hyperpmf" => Ok(HMath::hypergeometric_pmf(&args[0], &args[1], &args[2], &args[3])),
        "hypercdf" => Ok(HMath::hypergeometric_cdf(&args[0], &args[1], &args[2], &args[3])),
        "hypermean" => Ok(HMath::hypergeometric_mean(&args[0], &args[1], &args[2])),
        "hypervar" => Ok(HMath::hypergeometric_variance(&args[0], &args[1], &args[2])),
        "poissonpmf" => Ok(HMath::poisson_pmf(&args[0], &args[1])),
        "poissoncdf" => Ok(HMath::poisson_cdf(&args[0], &args[1])),
        "poissonmean" => Ok(HMath::poisson_mean(&args[0])),
        "poissonvar" => Ok(HMath::poisson_variance(&args[0])),

        // === Trigonometry ===
        "sin" => Ok(HMath::sin(&to_rad(&args[0]))),
        "cos" => Ok(HMath::cos(&to_rad(&args[0]))),
        "tan" => Ok(HMath::tan(&to_rad(&args[0]))),
        "cot" => Ok(HMath::cot(&to_rad(&args[0]))),
        "sec" => Ok(HMath::sec(&to_rad(&args[0]))),
        "csc" => Ok(HMath::csc(&to_rad(&args[0]))),
        "asin" => Ok(from_rad(HMath::asin(&args[0]))),
        "acos" => Ok(from_rad(HMath::acos(&args[0]))),
        "atan" => Ok(from_rad(HMath::atan(&args[0]))),
        "degrees" => Ok(HMath::degrees(&args[0])),
        "radians" => Ok(HMath::radians(&args[0])),

        // === Logic ===
        "mask" => Ok(HMath::mask(&args[0], &args[1])),
        "sgnext" => Ok(HMath::sgnext(&args[0], &args[1])),
        "not" => Ok(!args[0].clone()),
        "and" => {
            let mut result = args[0].clone();
            for a in &args[1..] {
                result = result & a.clone();
            }
            Ok(result)
        }
        "or" => {
            let mut result = args[0].clone();
            for a in &args[1..] {
                result = result | a.clone();
            }
            Ok(result)
        }
        "xor" => {
            let mut result = args[0].clone();
            for a in &args[1..] {
                result = result ^ a.clone();
            }
            Ok(result)
        }
        "shl" => Ok(args[0].clone() << args[1].clone()),
        "shr" => Ok(args[0].clone() >> args[1].clone()),
        "idiv" => Ok(HMath::idiv(&args[0], &args[1])),
        "mod" => Ok(args[0].clone() % args[1].clone()),

        // === Aliases (SpeedCrunch compatibility) ===
        "sign" => Ok(HMath::sgn(&args[0])),
        "unmask" => {
            let masked = HMath::mask(&args[0], &args[1]);
            Ok(!masked)
        }
        "poipmf" => Ok(HMath::poisson_pmf(&args[0], &args[1])),
        "poicdf" => Ok(HMath::poisson_cdf(&args[0], &args[1])),
        "poimean" => Ok(HMath::poisson_mean(&args[0])),
        "poivar" => Ok(HMath::poisson_variance(&args[0])),

        _ => Err(format!("Unknown function: {}", name)),
    };

    // Wrap successful results with no format override
    result.map(|v| (v, None))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::{HNumber, AngleMode};

    #[test]
    fn test_all_functions_non_empty() {
        let fns = all_functions();
        assert!(fns.len() >= 67, "Expected at least 67 functions, got {}", fns.len());
    }

    #[test]
    fn test_categories_present() {
        let cats = categories();
        assert!(cats.contains(&"Analysis"));
        assert!(cats.contains(&"Logarithm & Hyperbolic"));
        assert!(cats.contains(&"Discrete"));
        assert!(cats.contains(&"Probability"));
        assert!(cats.contains(&"Trigonometry"));
        assert!(cats.contains(&"Logic"));
    }

    #[test]
    fn test_all_functions_callable() {
        // Every 1-arg function should be callable with a simple value
        let fns = all_functions();
        for f in &fns {
            if f.argc_min == 1 && f.argc_max == 1 {
                let arg = if f.name == "arcosh" {
                    HNumber::from_f64(1.0) // arcosh domain: [1, inf)
                } else if f.name == "artanh" {
                    HNumber::from_f64(0.5) // artanh domain: (-1, 1)
                } else {
                    HNumber::from_f64(1.0)
                };
                let result = call_function(f.name, &[arg], AngleMode::Radian);
                assert!(result.is_ok(), "Function {}() failed: {:?}", f.name, result);
            }
        }
    }

    #[test]
    fn test_arity_validation() {
        // Too few args
        let r = call_function("gcd", &[HNumber::from_f64(1.0)], AngleMode::Radian);
        assert!(r.is_err());
        // Unknown function
        let r = call_function("notafunction", &[], AngleMode::Radian);
        assert!(r.is_err());
    }

    #[test]
    fn test_format_override_functions() {
        let n = HNumber::from_f64(42.0);
        let (_, fmt) = call_function("bin", &[n.clone()], AngleMode::Radian).unwrap();
        assert_eq!(fmt, Some(crate::math::NumberFormat::Binary));
        let (_, fmt) = call_function("hex", &[n.clone()], AngleMode::Radian).unwrap();
        assert_eq!(fmt, Some(crate::math::NumberFormat::Hexadecimal));
        let (_, fmt) = call_function("oct", &[n.clone()], AngleMode::Radian).unwrap();
        assert_eq!(fmt, Some(crate::math::NumberFormat::Octal));
        let (_, fmt) = call_function("dec", &[n], AngleMode::Radian).unwrap();
        assert_eq!(fmt, Some(crate::math::NumberFormat::General));
    }
}
