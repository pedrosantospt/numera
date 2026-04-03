// Numera Math Engine
// Exact decimal/rational arithmetic with guarded transcendental evaluation.

use num_bigint::BigInt;
use num_integer::Integer;
use num_rational::BigRational;
use num_traits::{Num, One, Signed, ToPrimitive, Zero};
use std::cmp::Ordering;
use std::fmt;
use std::ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Neg, Not, Rem, Shl, Shr, Sub};
use std::str::FromStr;
use std::sync::OnceLock;

/// Precision configuration
pub const DECPRECISION: u32 = 78;
pub const MAX_IO_DIGITS: u32 = 78;
pub const EVAL_PRECISION: u32 = DECPRECISION + 5;

const INTERNAL_DIGITS: usize = (EVAL_PRECISION as usize) + 8;
const SERIES_LIMIT: usize = 512;

/// Error codes for math operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MathError {
    Success,
    InvalidArg,
    Overflow,
    DivByZero,
    NotAnumber,
    OutOfDomain,
}

impl fmt::Display for MathError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MathError::Success => write!(f, ""),
            MathError::InvalidArg => write!(f, "invalid argument"),
            MathError::Overflow => write!(f, "overflow"),
            MathError::DivByZero => write!(f, "division by zero"),
            MathError::NotAnumber => write!(f, "not a number (NaN)"),
            MathError::OutOfDomain => write!(f, "out of domain"),
        }
    }
}

/// Number display format
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum NumberFormat {
    General,
    Fixed,
    Scientific,
    Engineering,
    Hexadecimal,
    Octal,
    Binary,
}

/// Angle mode for trigonometric functions
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum AngleMode {
    Radian,
    Degree,
}

#[derive(Clone, PartialEq, Eq)]
enum NumericValue {
    Real(BigRational),
    Complex { re: BigRational, im: BigRational },
    NaN,
}

/// High-precision number type
#[derive(Clone)]
/// High-precision number type supporting real, complex, and NaN values.
///
/// # Examples
///
/// ```
/// use numera::math::{HNumber, NumberFormat};
///
/// let a = HNumber::from_i64(7);
/// let b = HNumber::from_i64(3);
/// let sum = a + b;
/// assert_eq!(sum.format_with(NumberFormat::General, 15, '.'), "10");
/// ```
pub struct HNumber {
    value: NumericValue,
    error: MathError,
}

impl HNumber {
    pub fn new() -> Self {
        Self::from_real(BigRational::zero())
    }

    pub fn from_f64(v: f64) -> Self {
        if v.is_nan() {
            return Self::nan();
        }
        if v.is_infinite() {
            return Self::nan_with_error(MathError::Overflow);
        }
        match parse_decimal_rational(&format!("{:.17}", v)) {
            Ok(r) => Self::from_real(r),
            Err(_) => Self::nan_with_error(MathError::InvalidArg),
        }
    }

    pub fn from_i64(v: i64) -> Self {
        Self::from_real(BigRational::from_integer(BigInt::from(v)))
    }

    pub fn from_bigint(v: BigInt) -> Self {
        Self::from_real(BigRational::from_integer(v))
    }

    pub fn from_real(v: BigRational) -> Self {
        Self {
            value: NumericValue::Real(v),
            error: MathError::Success,
        }
    }

    pub fn from_complex(re: BigRational, im: BigRational) -> Self {
        if im.is_zero() {
            return Self::from_real(re);
        }
        Self {
            value: NumericValue::Complex { re, im },
            error: MathError::Success,
        }
    }

    pub fn nan() -> Self {
        Self {
            value: NumericValue::NaN,
            error: MathError::NotAnumber,
        }
    }

    pub fn nan_with_error(err: MathError) -> Self {
        Self {
            value: NumericValue::NaN,
            error: err,
        }
    }

    pub fn imaginary_unit() -> Self {
        Self::from_complex(BigRational::zero(), BigRational::one())
    }

    pub fn from_str_radix(s: &str) -> Result<Self, MathError> {
        let s = s.trim();
        if s.is_empty() {
            return Err(MathError::InvalidArg);
        }

        if let Some(value) = parse_complex_literal(s)? {
            return Ok(value);
        }

        if s.starts_with("0b") || s.starts_with("0B") {
            let digits = &s[2..];
            return match BigInt::from_str_radix(digits, 2) {
                Ok(v) => Ok(Self::from_bigint(v)),
                Err(_) => Err(MathError::InvalidArg),
            };
        }

        if s.starts_with("0o") || s.starts_with("0O") {
            let digits = &s[2..];
            return match BigInt::from_str_radix(digits, 8) {
                Ok(v) => Ok(Self::from_bigint(v)),
                Err(_) => Err(MathError::InvalidArg),
            };
        }

        if s.starts_with("0x") || s.starts_with("0X") {
            let digits = &s[2..];
            return match BigInt::from_str_radix(digits, 16) {
                Ok(v) => Ok(Self::from_bigint(v)),
                Err(_) => Err(MathError::InvalidArg),
            };
        }

        if let Some(digits) = s.strip_prefix('#') {
            return match BigInt::from_str_radix(digits, 16) {
                Ok(v) => Ok(Self::from_bigint(v)),
                Err(_) => Err(MathError::InvalidArg),
            };
        }

        parse_decimal_rational(s).map(Self::from_real)
    }

    pub fn value(&self) -> f64 {
        match &self.value {
            NumericValue::Real(v) => rational_to_f64(v),
            NumericValue::Complex { im, .. } if im.is_zero() => {
                if let NumericValue::Complex { re, .. } = &self.value {
                    rational_to_f64(re)
                } else {
                    f64::NAN
                }
            }
            _ => f64::NAN,
        }
    }

    pub fn is_nan(&self) -> bool {
        matches!(self.value, NumericValue::NaN)
    }

    pub fn is_zero(&self) -> bool {
        match &self.value {
            NumericValue::Real(v) => v.is_zero(),
            NumericValue::Complex { re, im } => re.is_zero() && im.is_zero(),
            NumericValue::NaN => false,
        }
    }

    pub fn real_value(&self) -> Option<&BigRational> {
        match &self.value {
            NumericValue::Real(v) => Some(v),
            NumericValue::Complex { re, im } if im.is_zero() => Some(re),
            _ => None,
        }
    }

    fn as_complex_parts(&self) -> Option<(BigRational, BigRational)> {
        match &self.value {
            NumericValue::Real(v) => Some((v.clone(), BigRational::zero())),
            NumericValue::Complex { re, im } => Some((re.clone(), im.clone())),
            NumericValue::NaN => None,
        }
    }

    /// Format number for display
    pub fn format_with(&self, fmt: NumberFormat, precision: i32, radix_char: char) -> String {
        if self.is_nan() {
            return "NaN".to_string();
        }

        let raw = match &self.value {
            NumericValue::Real(v) => format_real_value(v, fmt, precision),
            NumericValue::Complex { re, im } => format_complex_value(re, im, fmt, precision),
            NumericValue::NaN => "NaN".to_string(),
        };

        if radix_char != '.' {
            raw.replace('.', &radix_char.to_string())
        } else {
            raw
        }
    }
}

impl Default for HNumber {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for HNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format_with(NumberFormat::General, -1, '.'))
    }
}

impl fmt::Debug for HNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "HNumber({})", self)
    }
}

impl PartialEq for HNumber {
    fn eq(&self, other: &Self) -> bool {
        if self.is_nan() || other.is_nan() {
            return false;
        }
        self.value == other.value
    }
}

impl PartialOrd for HNumber {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self.real_value(), other.real_value()) {
            (Some(a), Some(b)) => a.partial_cmp(b),
            _ => None,
        }
    }
}

impl Add for HNumber {
    type Output = HNumber;

    fn add(self, rhs: HNumber) -> HNumber {
        binary_numeric_op(
            &self,
            &rhs,
            |a, b| a + b,
            |ar, ai, br, bi| (ar + br, ai + bi),
        )
    }
}

impl Sub for HNumber {
    type Output = HNumber;

    fn sub(self, rhs: HNumber) -> HNumber {
        binary_numeric_op(
            &self,
            &rhs,
            |a, b| a - b,
            |ar, ai, br, bi| (ar - br, ai - bi),
        )
    }
}

impl Mul for HNumber {
    type Output = HNumber;

    fn mul(self, rhs: HNumber) -> HNumber {
        binary_numeric_op(
            &self,
            &rhs,
            |a, b| a * b,
            |ar, ai, br, bi| {
                (
                    quantize(
                        &(ar.clone() * br.clone() - ai.clone() * bi.clone()),
                        INTERNAL_DIGITS,
                    ),
                    quantize(&(ar * bi + ai * br), INTERNAL_DIGITS),
                )
            },
        )
    }
}

impl Div for HNumber {
    type Output = HNumber;

    fn div(self, rhs: HNumber) -> HNumber {
        if rhs.is_zero() {
            return HNumber::nan_with_error(MathError::DivByZero);
        }

        binary_numeric_op(
            &self,
            &rhs,
            |a, b| a / b,
            |ar, ai, br, bi| {
                let denom = br.clone() * br.clone() + bi.clone() * bi.clone();
                if denom.is_zero() {
                    return (BigRational::zero(), BigRational::zero());
                }
                (
                    quantize(
                        &((ar.clone() * br.clone() + ai.clone() * bi.clone()) / denom.clone()),
                        INTERNAL_DIGITS,
                    ),
                    quantize(&((ai * br - ar * bi) / denom), INTERNAL_DIGITS),
                )
            },
        )
    }
}

impl Rem for HNumber {
    type Output = HNumber;

    fn rem(self, rhs: HNumber) -> HNumber {
        if rhs.is_zero() {
            return HNumber::nan_with_error(MathError::DivByZero);
        }

        match (self.real_value(), rhs.real_value()) {
            (Some(a), Some(b)) => {
                let q = integer_trunc(&(a / b));
                HNumber::from_real(a.clone() - b.clone() * BigRational::from_integer(q))
            }
            _ => HNumber::nan_with_error(MathError::InvalidArg),
        }
    }
}

impl Neg for HNumber {
    type Output = HNumber;

    fn neg(self) -> HNumber {
        match self.value {
            NumericValue::Real(v) => HNumber::from_real(-v),
            NumericValue::Complex { re, im } => HNumber::from_complex(-re, -im),
            NumericValue::NaN => HNumber::nan_with_error(self.error),
        }
    }
}

impl BitAnd for HNumber {
    type Output = HNumber;

    fn bitand(self, rhs: HNumber) -> HNumber {
        bitwise_op(self, rhs, |a, b| a & b)
    }
}

impl BitOr for HNumber {
    type Output = HNumber;

    fn bitor(self, rhs: HNumber) -> HNumber {
        bitwise_op(self, rhs, |a, b| a | b)
    }
}

impl BitXor for HNumber {
    type Output = HNumber;

    fn bitxor(self, rhs: HNumber) -> HNumber {
        bitwise_op(self, rhs, |a, b| a ^ b)
    }
}

impl Not for HNumber {
    type Output = HNumber;

    fn not(self) -> HNumber {
        match integer_pair(&self, &HNumber::from_i64(0)) {
            Ok((a, _)) => HNumber::from_i64((!a).to_i64().unwrap_or(0)),
            Err(err) => HNumber::nan_with_error(err),
        }
    }
}

impl Shl<HNumber> for HNumber {
    type Output = HNumber;

    fn shl(self, rhs: HNumber) -> HNumber {
        shift_op(self, rhs, true)
    }
}

impl Shr<HNumber> for HNumber {
    type Output = HNumber;

    fn shr(self, rhs: HNumber) -> HNumber {
        shift_op(self, rhs, false)
    }
}

/// Static math functions
pub struct HMath;

impl HMath {
    pub fn pi() -> HNumber {
        HNumber::from_real(pi_r().clone())
    }

    pub fn e() -> HNumber {
        HNumber::from_real(e_r().clone())
    }

    pub fn phi() -> HNumber {
        HNumber::from_real(phi_r().clone())
    }

    pub fn abs(x: &HNumber) -> HNumber {
        match x.as_complex_parts() {
            Some((re, im)) if im.is_zero() => HNumber::from_real(re.abs()),
            Some((re, im)) => {
                let magnitude_sq = re.clone() * re + im.clone() * im;
                HNumber::from_real(sqrt_real(&magnitude_sq, INTERNAL_DIGITS))
            }
            None => HNumber::nan(),
        }
    }

    pub fn integer(x: &HNumber) -> HNumber {
        match x.real_value() {
            Some(v) => HNumber::from_bigint(integer_trunc(v)),
            None => HNumber::nan_with_error(MathError::InvalidArg),
        }
    }

    pub fn frac(x: &HNumber) -> HNumber {
        match x.real_value() {
            Some(v) => HNumber::from_real(v.clone() - BigRational::from_integer(integer_trunc(v))),
            None => HNumber::nan_with_error(MathError::InvalidArg),
        }
    }

    pub fn floor(x: &HNumber) -> HNumber {
        match x.real_value() {
            Some(v) => HNumber::from_bigint(integer_floor(v)),
            None => HNumber::nan_with_error(MathError::InvalidArg),
        }
    }

    pub fn ceil(x: &HNumber) -> HNumber {
        match x.real_value() {
            Some(v) => HNumber::from_bigint(integer_ceil(v)),
            None => HNumber::nan_with_error(MathError::InvalidArg),
        }
    }

    pub fn round(x: &HNumber, decimals: Option<i32>) -> HNumber {
        match x.real_value() {
            Some(v) => HNumber::from_real(round_rational(v, decimals.unwrap_or(0))),
            None => HNumber::nan_with_error(MathError::InvalidArg),
        }
    }

    pub fn trunc(x: &HNumber, decimals: Option<i32>) -> HNumber {
        match x.real_value() {
            Some(v) => HNumber::from_real(trunc_rational(v, decimals.unwrap_or(0))),
            None => HNumber::nan_with_error(MathError::InvalidArg),
        }
    }

    pub fn sqrt(x: &HNumber) -> HNumber {
        match x.as_complex_parts() {
            Some((re, im)) if im.is_zero() && re.is_negative() => {
                HNumber::from_complex(BigRational::zero(), sqrt_real(&re.abs(), INTERNAL_DIGITS))
            }
            Some((re, im)) if im.is_zero() => HNumber::from_real(sqrt_real(&re, INTERNAL_DIGITS)),
            Some((re, im)) => complex_sqrt(&re, &im),
            None => HNumber::nan(),
        }
    }

    pub fn cbrt(x: &HNumber) -> HNumber {
        match x.real_value() {
            Some(v) => from_real_f64_approx(rational_to_f64(v).cbrt()),
            None => HNumber::nan_with_error(MathError::InvalidArg),
        }
    }

    pub fn raise(x: &HNumber, y: &HNumber) -> HNumber {
        if x.is_nan() || y.is_nan() {
            return HNumber::nan();
        }

        if let Some(exponent) = integer_value(y) {
            return pow_integer(x, exponent);
        }

        if let (Some(base), Some(exp)) = (x.real_value(), y.real_value()) {
            if base.is_negative() {
                if let Some(result) = raise_negative_real(base, exp) {
                    return result;
                }
                return HNumber::nan_with_error(MathError::OutOfDomain);
            }
        }

        if is_e_constant(x) {
            return Self::exp(y);
        }

        match (x.as_complex_parts(), y.as_complex_parts()) {
            (Some((xr, xi)), Some((yr, yi))) => {
                if yi.is_zero() && xi.is_zero() && xr.is_positive() {
                    let ln_x = ln_real(&xr, INTERNAL_DIGITS);
                    let real_part = quantize(&(yr.clone() * ln_x.clone()), INTERNAL_DIGITS);
                    let imag_part = quantize(&(yi * ln_x), INTERNAL_DIGITS);
                    complex_exp(&real_part, &imag_part)
                } else {
                    HNumber::nan_with_error(MathError::OutOfDomain)
                }
            }
            _ => HNumber::nan(),
        }
    }

    pub fn sgn(x: &HNumber) -> HNumber {
        if x.is_nan() {
            HNumber::nan()
        } else if let Some(v) = x.real_value() {
            if v.is_positive() {
                HNumber::from_i64(1)
            } else if v.is_negative() {
                HNumber::from_i64(-1)
            } else {
                HNumber::from_i64(0)
            }
        } else {
            HNumber::nan_with_error(MathError::InvalidArg)
        }
    }

    pub fn ln(x: &HNumber) -> HNumber {
        match x.as_complex_parts() {
            Some((re, im)) if im.is_zero() && re.is_positive() => {
                HNumber::from_real(ln_real(&re, INTERNAL_DIGITS))
            }
            Some((_re, im)) if im.is_zero() => HNumber::nan_with_error(MathError::OutOfDomain),
            Some((re, im)) => complex_ln(&re, &im),
            None => HNumber::nan(),
        }
    }

    pub fn log(x: &HNumber) -> HNumber {
        match Self::ln(x).real_value() {
            Some(v) => {
                HNumber::from_real(quantize(&(v.clone() / ln10_r().clone()), INTERNAL_DIGITS))
            }
            None => {
                let ln = Self::ln(x);
                if let Some((re, im)) = ln.as_complex_parts() {
                    HNumber::from_complex(
                        quantize(&(re / ln10_r().clone()), INTERNAL_DIGITS),
                        quantize(&(im / ln10_r().clone()), INTERNAL_DIGITS),
                    )
                } else {
                    HNumber::nan_with_error(MathError::OutOfDomain)
                }
            }
        }
    }

    pub fn lg(x: &HNumber) -> HNumber {
        match Self::ln(x).real_value() {
            Some(v) => {
                HNumber::from_real(quantize(&(v.clone() / ln2_r().clone()), INTERNAL_DIGITS))
            }
            None => HNumber::nan_with_error(MathError::OutOfDomain),
        }
    }

    pub fn exp(x: &HNumber) -> HNumber {
        match x.as_complex_parts() {
            Some((re, im)) if im.is_zero() => HNumber::from_real(exp_real(&re, INTERNAL_DIGITS)),
            Some((re, im)) => complex_exp(&re, &im),
            None => HNumber::nan(),
        }
    }

    pub fn sin(x: &HNumber) -> HNumber {
        match x.as_complex_parts() {
            Some((re, im)) if im.is_zero() => {
                HNumber::from_real(clean_small_real(sin_real(&re, INTERNAL_DIGITS)))
            }
            Some((re, im)) => {
                let sin_re = sin_real(&re, INTERNAL_DIGITS);
                let cos_re = cos_real(&re, INTERNAL_DIGITS);
                let sinh_im = from_f64_to_rational(rational_to_f64(&im).sinh());
                let cosh_im = from_f64_to_rational(rational_to_f64(&im).cosh());
                HNumber::from_complex(
                    quantize(&(sin_re * cosh_im.clone()), INTERNAL_DIGITS),
                    quantize(&(cos_re * sinh_im), INTERNAL_DIGITS),
                )
            }
            None => HNumber::nan(),
        }
    }

    pub fn cos(x: &HNumber) -> HNumber {
        match x.as_complex_parts() {
            Some((re, im)) if im.is_zero() => {
                HNumber::from_real(clean_small_real(cos_real(&re, INTERNAL_DIGITS)))
            }
            Some((re, im)) => {
                let sin_re = sin_real(&re, INTERNAL_DIGITS);
                let cos_re = cos_real(&re, INTERNAL_DIGITS);
                let sinh_im = from_f64_to_rational(rational_to_f64(&im).sinh());
                let cosh_im = from_f64_to_rational(rational_to_f64(&im).cosh());
                HNumber::from_complex(
                    quantize(&(cos_re * cosh_im.clone()), INTERNAL_DIGITS),
                    quantize(&(-sin_re * sinh_im), INTERNAL_DIGITS),
                )
            }
            None => HNumber::nan(),
        }
    }

    pub fn tan(x: &HNumber) -> HNumber {
        let cosine = Self::cos(x);
        if cosine.real_value().map(is_near_zero).unwrap_or(false) {
            HNumber::nan_with_error(MathError::OutOfDomain)
        } else {
            Self::sin(x) / cosine
        }
    }

    pub fn cot(x: &HNumber) -> HNumber {
        let sine = Self::sin(x);
        if sine.real_value().map(is_near_zero).unwrap_or(false) {
            HNumber::nan_with_error(MathError::OutOfDomain)
        } else {
            Self::cos(x) / sine
        }
    }

    pub fn sec(x: &HNumber) -> HNumber {
        let cosine = Self::cos(x);
        if cosine.real_value().map(is_near_zero).unwrap_or(false) {
            HNumber::nan_with_error(MathError::OutOfDomain)
        } else {
            HNumber::from_i64(1) / cosine
        }
    }

    pub fn csc(x: &HNumber) -> HNumber {
        let sine = Self::sin(x);
        if sine.real_value().map(is_near_zero).unwrap_or(false) {
            HNumber::nan_with_error(MathError::OutOfDomain)
        } else {
            HNumber::from_i64(1) / sine
        }
    }

    pub fn asin(x: &HNumber) -> HNumber {
        unary_f64_real(x, |v| v.asin(), MathError::OutOfDomain)
    }

    pub fn acos(x: &HNumber) -> HNumber {
        unary_f64_real(x, |v| v.acos(), MathError::OutOfDomain)
    }

    pub fn atan(x: &HNumber) -> HNumber {
        unary_f64_real(x, |v| v.atan(), MathError::InvalidArg)
    }

    pub fn sinh(x: &HNumber) -> HNumber {
        unary_f64_real(x, |v| v.sinh(), MathError::InvalidArg)
    }

    pub fn cosh(x: &HNumber) -> HNumber {
        unary_f64_real(x, |v| v.cosh(), MathError::InvalidArg)
    }

    pub fn tanh(x: &HNumber) -> HNumber {
        unary_f64_real(x, |v| v.tanh(), MathError::InvalidArg)
    }

    pub fn arsinh(x: &HNumber) -> HNumber {
        unary_f64_real(x, |v| v.asinh(), MathError::InvalidArg)
    }

    pub fn arcosh(x: &HNumber) -> HNumber {
        unary_f64_real(x, |v| v.acosh(), MathError::OutOfDomain)
    }

    pub fn artanh(x: &HNumber) -> HNumber {
        unary_f64_real(x, |v| v.atanh(), MathError::OutOfDomain)
    }

    pub fn degrees(x: &HNumber) -> HNumber {
        x.clone()
            * HNumber::from_real(quantize(
                &(rat_from_i64(180) / pi_r().clone()),
                INTERNAL_DIGITS,
            ))
    }

    pub fn radians(x: &HNumber) -> HNumber {
        x.clone()
            * HNumber::from_real(quantize(
                &(pi_r().clone() / rat_from_i64(180)),
                INTERNAL_DIGITS,
            ))
    }

    pub fn factorial(x: &HNumber) -> HNumber {
        if x.is_nan() {
            return HNumber::nan();
        }

        if let Some(v) = x.real_value() {
            if v.is_integer() {
                let n = integer_trunc(v);
                if n.is_negative() {
                    return HNumber::nan_with_error(MathError::OutOfDomain);
                }
                let n_u64 = match n.to_u64() {
                    Some(v) => v,
                    None => return HNumber::nan_with_error(MathError::Overflow),
                };
                let mut result = BigInt::one();
                for i in 2..=n_u64 {
                    result *= i;
                }
                return HNumber::from_bigint(result);
            }
            return Self::gamma(&(x.clone() + HNumber::from_i64(1)));
        }

        HNumber::nan_with_error(MathError::InvalidArg)
    }

    pub fn gamma(x: &HNumber) -> HNumber {
        match x.real_value() {
            Some(v) => gamma_real(v),
            None => HNumber::nan_with_error(MathError::InvalidArg),
        }
    }

    pub fn lngamma(x: &HNumber) -> HNumber {
        let g = Self::gamma(x);
        if g.is_nan() {
            return g;
        }
        Self::ln(&Self::abs(&g))
    }

    #[allow(non_snake_case)]
    pub fn nCr(n: &HNumber, r: &HNumber) -> HNumber {
        combinatoric(n, r, false)
    }

    #[allow(non_snake_case)]
    pub fn nPr(n: &HNumber, r: &HNumber) -> HNumber {
        combinatoric(n, r, true)
    }

    pub fn gcd(a: &HNumber, b: &HNumber) -> HNumber {
        match integer_pair(a, b) {
            Ok((mut x, mut y)) => {
                x = x.abs();
                y = y.abs();
                while !y.is_zero() {
                    let t = y.clone();
                    y = x % t.clone();
                    x = t;
                }
                HNumber::from_bigint(x)
            }
            Err(err) => HNumber::nan_with_error(err),
        }
    }

    pub fn idiv(a: &HNumber, b: &HNumber) -> HNumber {
        match (a.real_value(), b.real_value()) {
            (Some(ar), Some(br)) if !br.is_zero() => {
                HNumber::from_bigint(integer_trunc(&(ar.clone() / br.clone())))
            }
            (Some(_), Some(_)) => HNumber::nan_with_error(MathError::DivByZero),
            _ => HNumber::nan_with_error(MathError::InvalidArg),
        }
    }

    pub fn erf(x: &HNumber) -> HNumber {
        unary_f64_real(x, erf_impl, MathError::InvalidArg)
    }

    pub fn erfc(x: &HNumber) -> HNumber {
        unary_f64_real(x, |v| 1.0 - erf_impl(v), MathError::InvalidArg)
    }

    pub fn mask(x: &HNumber, bits: &HNumber) -> HNumber {
        match integer_pair(x, bits) {
            Ok((xv, bv)) => {
                let b = match bv.to_u32() {
                    Some(v) if v <= 64 => v,
                    _ => return HNumber::nan_with_error(MathError::InvalidArg),
                };
                let mask = if b == 64 { u64::MAX } else { (1u64 << b) - 1 };
                HNumber::from_i64(((xv.to_i64().unwrap_or(0) as u64) & mask) as i64)
            }
            Err(err) => HNumber::nan_with_error(err),
        }
    }

    pub fn sgnext(x: &HNumber, bits: &HNumber) -> HNumber {
        match integer_pair(x, bits) {
            Ok((xv, bv)) => {
                let b = match bv.to_u32() {
                    Some(v) if (1..=64).contains(&v) => v,
                    _ => return HNumber::nan_with_error(MathError::InvalidArg),
                };
                let val = xv.to_i64().unwrap_or(0);
                let sign_bit = 1i64 << (b - 1);
                let mask = if b == 64 { -1i64 } else { (1i64 << b) - 1 };
                let masked = val & mask;
                let result = if masked & sign_bit != 0 {
                    masked | !mask
                } else {
                    masked
                };
                HNumber::from_i64(result)
            }
            Err(err) => HNumber::nan_with_error(err),
        }
    }

    pub fn binomial_pmf(k: &HNumber, n: &HNumber, p: &HNumber) -> HNumber {
        let comb = Self::nCr(n, k);
        let pk = Self::raise(p, k);
        let qnk = Self::raise(
            &(HNumber::from_i64(1) - p.clone()),
            &(n.clone() - k.clone()),
        );
        comb * pk * qnk
    }

    pub fn binomial_cdf(k: &HNumber, n: &HNumber, p: &HNumber) -> HNumber {
        let Some(ki) = integer_value(k).and_then(|v| v.to_i64()) else {
            return HNumber::nan_with_error(MathError::InvalidArg);
        };
        let mut sum = HNumber::from_i64(0);
        for i in 0..=ki {
            sum = sum + Self::binomial_pmf(&HNumber::from_i64(i), n, p);
        }
        sum
    }

    pub fn binomial_mean(n: &HNumber, p: &HNumber) -> HNumber {
        n.clone() * p.clone()
    }

    pub fn binomial_variance(n: &HNumber, p: &HNumber) -> HNumber {
        n.clone() * p.clone() * (HNumber::from_i64(1) - p.clone())
    }

    pub fn poisson_pmf(k: &HNumber, lambda: &HNumber) -> HNumber {
        let numerator = Self::exp(&(-lambda.clone())) * Self::raise(lambda, k);
        numerator / Self::factorial(k)
    }

    pub fn poisson_cdf(k: &HNumber, lambda: &HNumber) -> HNumber {
        let Some(ki) = integer_value(k).and_then(|v| v.to_i64()) else {
            return HNumber::nan_with_error(MathError::InvalidArg);
        };
        let mut sum = HNumber::from_i64(0);
        for i in 0..=ki {
            sum = sum + Self::poisson_pmf(&HNumber::from_i64(i), lambda);
        }
        sum
    }

    pub fn poisson_mean(lambda: &HNumber) -> HNumber {
        lambda.clone()
    }

    pub fn poisson_variance(lambda: &HNumber) -> HNumber {
        lambda.clone()
    }

    pub fn hypergeometric_pmf(k: &HNumber, nn: &HNumber, m: &HNumber, n: &HNumber) -> HNumber {
        let c1 = Self::nCr(m, k);
        let c2 = Self::nCr(&(nn.clone() - m.clone()), &(n.clone() - k.clone()));
        let c3 = Self::nCr(nn, n);
        c1 * c2 / c3
    }

    pub fn hypergeometric_cdf(k: &HNumber, nn: &HNumber, m: &HNumber, n: &HNumber) -> HNumber {
        let Some(ki) = integer_value(k).and_then(|v| v.to_i64()) else {
            return HNumber::nan_with_error(MathError::InvalidArg);
        };
        let mut sum = HNumber::from_i64(0);
        for i in 0..=ki {
            sum = sum + Self::hypergeometric_pmf(&HNumber::from_i64(i), nn, m, n);
        }
        sum
    }

    pub fn hypergeometric_mean(nn: &HNumber, m: &HNumber, n: &HNumber) -> HNumber {
        n.clone() * m.clone() / nn.clone()
    }

    pub fn hypergeometric_variance(nn: &HNumber, m: &HNumber, n: &HNumber) -> HNumber {
        let term1 = n.clone() * m.clone() / nn.clone();
        let term2 = HNumber::from_i64(1) - m.clone() / nn.clone();
        let term3 = (nn.clone() - n.clone()) / (nn.clone() - HNumber::from_i64(1));
        term1 * term2 * term3
    }
}

fn binary_numeric_op<FReal, FComplex>(
    lhs: &HNumber,
    rhs: &HNumber,
    real_op: FReal,
    complex_op: FComplex,
) -> HNumber
where
    FReal: Fn(BigRational, BigRational) -> BigRational,
    FComplex: Fn(BigRational, BigRational, BigRational, BigRational) -> (BigRational, BigRational),
{
    match (lhs.as_complex_parts(), rhs.as_complex_parts()) {
        (Some((lr, li)), Some((rr, ri))) if li.is_zero() && ri.is_zero() => {
            HNumber::from_real(real_op(lr, rr))
        }
        (Some((lr, li)), Some((rr, ri))) => {
            let (re, im) = complex_op(lr, li, rr, ri);
            HNumber::from_complex(re, im)
        }
        _ => HNumber::nan(),
    }
}

fn bitwise_op<F>(lhs: HNumber, rhs: HNumber, op: F) -> HNumber
where
    F: Fn(i64, i64) -> i64,
{
    match integer_pair(&lhs, &rhs) {
        Ok((a, b)) => HNumber::from_i64(op(a.to_i64().unwrap_or(0), b.to_i64().unwrap_or(0))),
        Err(err) => HNumber::nan_with_error(err),
    }
}

fn shift_op(lhs: HNumber, rhs: HNumber, left: bool) -> HNumber {
    match integer_pair(&lhs, &rhs) {
        Ok((a, b)) => {
            let lhs_i = a.to_i64().unwrap_or(0);
            let rhs_u = match b.to_u32() {
                Some(v) => v,
                None => return HNumber::nan_with_error(MathError::InvalidArg),
            };
            if left {
                HNumber::from_i64(lhs_i.wrapping_shl(rhs_u))
            } else {
                HNumber::from_i64(lhs_i.wrapping_shr(rhs_u))
            }
        }
        Err(err) => HNumber::nan_with_error(err),
    }
}

fn integer_pair(a: &HNumber, b: &HNumber) -> Result<(BigInt, BigInt), MathError> {
    let Some(av) = a.real_value() else {
        return Err(MathError::InvalidArg);
    };
    let Some(bv) = b.real_value() else {
        return Err(MathError::InvalidArg);
    };
    if !av.is_integer() || !bv.is_integer() {
        return Err(MathError::InvalidArg);
    }
    Ok((integer_trunc(av), integer_trunc(bv)))
}

fn integer_value(x: &HNumber) -> Option<BigInt> {
    x.real_value().filter(|v| v.is_integer()).map(integer_trunc)
}

fn rat_from_i64(v: i64) -> BigRational {
    BigRational::from_integer(BigInt::from(v))
}

fn rational_to_f64(v: &BigRational) -> f64 {
    v.to_f64().unwrap_or_else(|| {
        if v.is_negative() {
            f64::NEG_INFINITY
        } else {
            f64::INFINITY
        }
    })
}

fn parse_decimal_rational(s: &str) -> Result<BigRational, MathError> {
    let s = s.trim();
    if s.is_empty() {
        return Err(MathError::InvalidArg);
    }

    let (mantissa, exponent) = if let Some(pos) = s.find(['e', 'E']) {
        let exp = i64::from_str(&s[pos + 1..]).map_err(|_| MathError::InvalidArg)?;
        (&s[..pos], exp)
    } else {
        (s, 0)
    };

    let negative = mantissa.starts_with('-');
    let unsigned = mantissa.strip_prefix(['+', '-']).unwrap_or(mantissa);
    let mut digits = String::new();
    let mut frac_len = 0usize;
    let mut seen_dot = false;

    for ch in unsigned.chars() {
        match ch {
            '_' => {}
            '.' => {
                if seen_dot {
                    return Err(MathError::InvalidArg);
                }
                seen_dot = true;
            }
            d if d.is_ascii_digit() => {
                digits.push(d);
                if seen_dot {
                    frac_len += 1;
                }
            }
            _ => return Err(MathError::InvalidArg),
        }
    }

    if digits.is_empty() {
        return Err(MathError::InvalidArg);
    }

    let mut numerator = BigInt::from_str(&digits).map_err(|_| MathError::InvalidArg)?;
    if negative {
        numerator = -numerator;
    }

    let scale = frac_len as i64 - exponent;
    if scale >= 0 {
        Ok(BigRational::new(numerator, pow10(scale as usize)))
    } else {
        Ok(BigRational::from_integer(
            numerator * pow10((-scale) as usize),
        ))
    }
}

fn parse_complex_literal(s: &str) -> Result<Option<HNumber>, MathError> {
    let normalized = s.trim();
    if !normalized.ends_with('i') && !normalized.ends_with('j') {
        return Ok(None);
    }

    let body = &normalized[..normalized.len() - 1];
    if body.is_empty() || body == "+" {
        return Ok(Some(HNumber::imaginary_unit()));
    }
    if body == "-" {
        return Ok(Some(HNumber::from_complex(
            BigRational::zero(),
            -BigRational::one(),
        )));
    }

    if let Some(split) = find_complex_split(body) {
        let re = parse_decimal_rational(&body[..split])?;
        let im = parse_decimal_rational(&body[split..])?;
        return Ok(Some(HNumber::from_complex(re, im)));
    }

    let im = parse_decimal_rational(body)?;
    Ok(Some(HNumber::from_complex(BigRational::zero(), im)))
}

fn find_complex_split(body: &str) -> Option<usize> {
    let bytes = body.as_bytes();
    for i in (1..bytes.len()).rev() {
        let ch = bytes[i] as char;
        if (ch == '+' || ch == '-') && bytes[i - 1] as char != 'e' && bytes[i - 1] as char != 'E' {
            return Some(i);
        }
    }
    None
}

fn format_real_value(v: &BigRational, fmt: NumberFormat, precision: i32) -> String {
    match fmt {
        NumberFormat::Hexadecimal => format_integer_base(v, 16, "0x"),
        NumberFormat::Octal => format_integer_base(v, 8, "0o"),
        NumberFormat::Binary => format_integer_base(v, 2, "0b"),
        NumberFormat::Scientific => format_scientific(v, precision, false),
        NumberFormat::Engineering => format_scientific(v, precision, true),
        NumberFormat::Fixed => {
            let digits = if precision < 0 {
                MAX_IO_DIGITS as usize
            } else {
                precision as usize
            };
            rational_to_decimal_string(v, digits, precision >= 0)
        }
        NumberFormat::General => {
            if v.is_zero() {
                "0".to_string()
            } else {
                let exp = decimal_exponent_estimate(v);
                if !(-4..15).contains(&exp) {
                    format_scientific(v, if precision < 0 { 18 } else { precision }, false)
                } else {
                    rational_to_decimal_string(
                        v,
                        if precision < 0 {
                            MAX_IO_DIGITS as usize
                        } else {
                            precision as usize
                        },
                        false,
                    )
                }
            }
        }
    }
}

fn format_complex_value(
    re: &BigRational,
    im: &BigRational,
    fmt: NumberFormat,
    precision: i32,
) -> String {
    let threshold = epsilon((MAX_IO_DIGITS as usize).saturating_sub(8));
    let re = if re.abs() < threshold {
        BigRational::zero()
    } else {
        re.clone()
    };
    let im = if im.abs() < threshold {
        BigRational::zero()
    } else {
        im.clone()
    };

    if im.is_zero() {
        return format_real_value(&re, fmt, precision);
    }

    let re_zero = re.is_zero();
    let im_abs = im.abs();
    let im_text = if im_abs == BigRational::one() {
        "i".to_string()
    } else {
        format!(
            "{}i",
            format_real_value(&im_abs, NumberFormat::General, precision)
        )
    };

    if re_zero {
        if im.is_negative() {
            format!("-{}", im_text)
        } else {
            im_text
        }
    } else {
        let re_text = format_real_value(&re, NumberFormat::General, precision);
        if im.is_negative() {
            format!("{}-{}", re_text, im_text)
        } else {
            format!("{}+{}", re_text, im_text)
        }
    }
}

fn format_integer_base(v: &BigRational, radix: u32, prefix: &str) -> String {
    if !v.is_integer() {
        return rational_to_decimal_string(v, MAX_IO_DIGITS as usize, false);
    }
    let n = integer_trunc(v);
    let abs = n.abs().to_str_radix(radix);
    if n.is_negative() {
        format!("-{}{}", prefix, abs)
    } else {
        format!("{}{}", prefix, abs)
    }
}

fn rational_to_decimal_string(v: &BigRational, frac_digits: usize, fixed: bool) -> String {
    let negative = v.is_negative();
    let mut numer = v.numer().abs();
    let denom = v.denom().clone();
    let mut integer = (&numer / &denom).to_string();
    numer %= &denom;

    if frac_digits == 0 {
        let mut result = integer;
        if negative && result != "0" {
            result.insert(0, '-');
        }
        return result;
    }

    let mut digits: Vec<u8> = Vec::with_capacity(frac_digits + 1);
    let mut remainder = numer;
    for _ in 0..=frac_digits {
        remainder *= 10;
        let digit = (&remainder / &denom).to_u8().unwrap_or(0);
        remainder %= &denom;
        digits.push(digit);
    }

    let round_up = digits.pop().unwrap_or(0) >= 5;
    if round_up {
        let mut idx = digits.len();
        while idx > 0 {
            idx -= 1;
            if digits[idx] < 9 {
                digits[idx] += 1;
                break;
            }
            digits[idx] = 0;
        }
        if idx == 0 && digits.first() == Some(&0) {
            let integer_big: BigInt =
                BigInt::from_str(&integer).unwrap_or_else(|_| BigInt::zero()) + 1;
            integer = integer_big.to_string();
        }
    }

    let mut frac = digits
        .into_iter()
        .map(|d| char::from(b'0' + d))
        .collect::<String>();
    if !fixed {
        while frac.ends_with('0') {
            frac.pop();
        }
    }

    let mut result = if frac.is_empty() {
        integer
    } else {
        format!("{}.{}", integer, frac)
    };

    if negative && result != "0" {
        result.insert(0, '-');
    }
    result
}

fn format_scientific(v: &BigRational, precision: i32, engineering: bool) -> String {
    if v.is_zero() {
        return "0".to_string();
    }
    let mut exp = decimal_exponent_estimate(v);
    if engineering {
        exp = Integer::div_floor(&exp, &3) * 3;
    }
    let scaled = quantize(&(v.clone() / pow10_r(exp)), INTERNAL_DIGITS);
    let mantissa = rational_to_decimal_string(
        &scaled,
        if precision < 0 {
            18
        } else {
            precision as usize
        },
        false,
    );
    format!("{}e{}", mantissa, exp)
}

fn decimal_exponent_estimate(v: &BigRational) -> i64 {
    if v.is_zero() {
        return 0;
    }

    let mut exp = v.numer().abs().to_string().len() as i64 - v.denom().to_string().len() as i64;
    let ten = rat_from_i64(10);
    let one = BigRational::one();
    let mut scaled = v.abs() / pow10_r(exp);

    while scaled >= ten {
        scaled /= ten.clone();
        exp += 1;
    }
    while scaled < one {
        scaled *= ten.clone();
        exp -= 1;
    }

    exp
}

fn pow10(exp: usize) -> BigInt {
    BigInt::from(10u8).pow(exp as u32)
}

fn pow10_r(exp: i64) -> BigRational {
    if exp >= 0 {
        BigRational::from_integer(pow10(exp as usize))
    } else {
        BigRational::new(BigInt::one(), pow10((-exp) as usize))
    }
}

fn integer_trunc(v: &BigRational) -> BigInt {
    v.numer() / v.denom()
}

fn integer_floor(v: &BigRational) -> BigInt {
    let q = v.numer() / v.denom();
    let r = v.numer() % v.denom();
    if v.is_negative() && !r.is_zero() {
        q - 1
    } else {
        q
    }
}

fn integer_ceil(v: &BigRational) -> BigInt {
    let q = v.numer() / v.denom();
    let r = v.numer() % v.denom();
    if v.is_positive() && !r.is_zero() {
        q + 1
    } else {
        q
    }
}

fn round_rational(v: &BigRational, decimals: i32) -> BigRational {
    let scale = pow10_r(decimals as i64);
    let scaled = v.clone() * scale.clone();
    let rounded = round_to_int(&scaled);
    BigRational::from_integer(rounded) / scale
}

fn trunc_rational(v: &BigRational, decimals: i32) -> BigRational {
    let scale = pow10_r(decimals as i64);
    let scaled = v.clone() * scale.clone();
    BigRational::from_integer(integer_trunc(&scaled)) / scale
}

fn round_to_int(v: &BigRational) -> BigInt {
    let abs = v.abs();
    let floor = integer_floor(&abs);
    let frac = abs - BigRational::from_integer(floor.clone());
    let mut rounded = floor;
    if frac >= BigRational::new(BigInt::one(), BigInt::from(2)) {
        rounded += 1;
    }
    if v.is_negative() {
        -rounded
    } else {
        rounded
    }
}

fn quantize(v: &BigRational, digits: usize) -> BigRational {
    if digits == 0 {
        return BigRational::from_integer(round_to_int(v));
    }
    let scale = pow10(digits);
    let scaled = v.clone() * BigRational::from_integer(scale.clone());
    BigRational::new(round_to_int(&scaled), scale)
}

fn epsilon(digits: usize) -> BigRational {
    BigRational::new(BigInt::one(), pow10(digits.max(1)))
}

fn is_near_zero(v: &BigRational) -> bool {
    v.abs() < epsilon((MAX_IO_DIGITS as usize).saturating_sub(8))
}

fn clean_small_real(v: BigRational) -> BigRational {
    if is_near_zero(&v) {
        BigRational::zero()
    } else {
        v
    }
}

fn sqrt_real(x: &BigRational, digits: usize) -> BigRational {
    if x.is_zero() {
        return BigRational::zero();
    }

    let exp = decimal_exponent_estimate(x);
    let mut guess = pow10_r(Integer::div_floor(&(exp + 1), &2));
    let two = rat_from_i64(2);
    let tol = epsilon(digits);

    for _ in 0..128 {
        let next = quantize(
            &((guess.clone() + x.clone() / guess.clone()) / two.clone()),
            digits + 4,
        );
        let delta = (next.clone() - guess.clone()).abs();
        guess = next;
        if delta < tol {
            break;
        }
    }

    quantize(&guess, digits)
}

fn exp_real(x: &BigRational, digits: usize) -> BigRational {
    if x.is_zero() {
        return BigRational::one();
    }
    if x.is_negative() {
        return quantize(
            &(BigRational::one() / exp_real(&x.abs(), digits + 4)),
            digits,
        );
    }

    let mut halvings = 0usize;
    let mut reduced = x.clone();
    while rational_to_f64(&reduced).abs() > 0.5 {
        reduced /= rat_from_i64(2);
        halvings += 1;
    }

    let tol = epsilon(digits + 2);
    let mut sum = BigRational::one();
    let mut term = BigRational::one();

    for n in 1..SERIES_LIMIT {
        term = quantize(
            &(term * reduced.clone() / rat_from_i64(n as i64)),
            digits + 6,
        );
        sum += term.clone();
        if term.abs() < tol {
            break;
        }
    }

    let mut result = quantize(&sum, digits + 6);
    for _ in 0..halvings {
        result = quantize(&(result.clone() * result), digits + 6);
    }
    quantize(&result, digits)
}

fn ln_real(x: &BigRational, digits: usize) -> BigRational {
    let one = BigRational::one();
    let mut exp10 = decimal_exponent_estimate(x);
    let mut normalized = quantize(&(x.clone() / pow10_r(exp10)), digits + 8);

    while normalized >= rat_from_i64(10) {
        normalized /= rat_from_i64(10);
        exp10 += 1;
    }
    while normalized < one {
        normalized *= rat_from_i64(10);
        exp10 -= 1;
    }

    let upper = BigRational::new(BigInt::from(11), BigInt::from(10));
    let lower = BigRational::new(BigInt::from(9), BigInt::from(10));
    let mut factor = BigInt::one();
    while normalized > upper || normalized < lower {
        normalized = sqrt_real(&normalized, digits + 8);
        factor *= 2;
    }

    let t = quantize(
        &((normalized.clone() - one.clone()) / (normalized.clone() + one.clone())),
        digits + 8,
    );
    let t2 = quantize(&(t.clone() * t.clone()), digits + 8);
    let tol = epsilon(digits + 3);
    let mut sum = BigRational::zero();
    let mut term = t.clone();
    let mut denom = 1i64;

    for _ in 0..SERIES_LIMIT {
        let addend = quantize(&(term.clone() / rat_from_i64(denom)), digits + 8);
        sum += addend.clone();
        if addend.abs() < tol {
            break;
        }
        term = quantize(&(term * t2.clone()), digits + 8);
        denom += 2;
    }

    let ln_normalized = quantize(
        &(rat_from_i64(2) * sum * BigRational::from_integer(factor)),
        digits + 8,
    );
    quantize(
        &(ln_normalized + ln10_r().clone() * rat_from_i64(exp10)),
        digits,
    )
}

fn sin_real(x: &BigRational, digits: usize) -> BigRational {
    let reduced = reduce_angle(x);
    let tol = epsilon(digits + 2);
    let mut sum = reduced.clone();
    let mut term = reduced.clone();
    let x2 = quantize(&(reduced.clone() * reduced.clone()), digits + 8);

    for n in 1..SERIES_LIMIT {
        let denom = rat_from_i64((2 * n * (2 * n + 1)) as i64);
        term = quantize(&(-term * x2.clone() / denom), digits + 8);
        sum += term.clone();
        if term.abs() < tol {
            break;
        }
    }

    quantize(&sum, digits)
}

fn cos_real(x: &BigRational, digits: usize) -> BigRational {
    let reduced = reduce_angle(x);
    let tol = epsilon(digits + 2);
    let mut sum = BigRational::one();
    let mut term = BigRational::one();
    let x2 = quantize(&(reduced.clone() * reduced.clone()), digits + 8);

    for n in 1..SERIES_LIMIT {
        let denom = rat_from_i64(((2 * n - 1) * (2 * n)) as i64);
        term = quantize(&(-term * x2.clone() / denom), digits + 8);
        sum += term.clone();
        if term.abs() < tol {
            break;
        }
    }

    quantize(&sum, digits)
}

fn reduce_angle(x: &BigRational) -> BigRational {
    let tau = two_pi_r().clone();
    let quotient = round_to_int(&(x.clone() / tau.clone()));
    quantize(
        &(x.clone() - tau * BigRational::from_integer(quotient)),
        INTERNAL_DIGITS,
    )
}

fn complex_exp(re: &BigRational, im: &BigRational) -> HNumber {
    let ea = exp_real(re, INTERNAL_DIGITS);
    let cos_b = cos_real(im, INTERNAL_DIGITS);
    let sin_b = sin_real(im, INTERNAL_DIGITS);
    HNumber::from_complex(
        quantize(&(ea.clone() * cos_b), INTERNAL_DIGITS),
        quantize(&(ea * sin_b), INTERNAL_DIGITS),
    )
}

fn complex_ln(re: &BigRational, im: &BigRational) -> HNumber {
    let modulus = sqrt_real(
        &(re.clone() * re.clone() + im.clone() * im.clone()),
        INTERNAL_DIGITS,
    );
    let arg = rational_to_f64(im).atan2(rational_to_f64(re));
    HNumber::from_complex(
        ln_real(&modulus, INTERNAL_DIGITS),
        from_f64_to_rational(arg),
    )
}

fn complex_sqrt(re: &BigRational, im: &BigRational) -> HNumber {
    let magnitude = sqrt_real(
        &(re.clone() * re.clone() + im.clone() * im.clone()),
        INTERNAL_DIGITS,
    );
    let two = rat_from_i64(2);
    let real = sqrt_real(
        &quantize(
            &((magnitude.clone() + re.clone()) / two.clone()),
            INTERNAL_DIGITS,
        ),
        INTERNAL_DIGITS,
    );
    let imag_mag = sqrt_real(
        &quantize(&((magnitude - re.clone()) / two), INTERNAL_DIGITS),
        INTERNAL_DIGITS,
    );
    let imag = if im.is_negative() {
        -imag_mag
    } else {
        imag_mag
    };
    HNumber::from_complex(real, imag)
}

fn raise_negative_real(base: &BigRational, exp: &BigRational) -> Option<HNumber> {
    let numer = exp.numer().clone();
    let denom = exp.denom().clone();
    let denom_i64 = denom.to_i64()?;

    if denom_i64 % 2 == 0 {
        return None;
    }

    let abs_base = base.abs();
    let exponent_abs = BigRational::new(numer.abs(), denom);
    let approx = rational_to_f64(&abs_base).powf(rational_to_f64(&exponent_abs));
    if !approx.is_finite() {
        return Some(HNumber::nan_with_error(MathError::Overflow));
    }

    let approx_r = from_f64_to_rational(approx);
    let snapped = snap_near_integer(&approx_r);
    let signed = if numer.is_odd() { -snapped } else { snapped };
    Some(HNumber::from_real(signed))
}

fn snap_near_integer(v: &BigRational) -> BigRational {
    let nearest = round_to_int(v);
    let nearest_r = BigRational::from_integer(nearest);
    if (v.clone() - nearest_r.clone()).abs() < epsilon(12) {
        nearest_r
    } else {
        v.clone()
    }
}

fn pow_integer(base: &HNumber, exponent: BigInt) -> HNumber {
    if exponent.is_zero() {
        return HNumber::from_i64(1);
    }
    let negative = exponent.is_negative();
    let mut exp = exponent.abs();
    let mut result = HNumber::from_i64(1);
    let mut factor = base.clone();

    while !exp.is_zero() {
        if exp.is_odd() {
            result = quantized_number(&(result * factor.clone()), INTERNAL_DIGITS);
        }
        exp >>= 1usize;
        if !exp.is_zero() {
            factor = quantized_number(&(factor.clone() * factor), INTERNAL_DIGITS);
        }
    }

    if negative {
        HNumber::from_i64(1) / result
    } else {
        result
    }
}

fn quantized_number(v: &HNumber, digits: usize) -> HNumber {
    match v.as_complex_parts() {
        Some((re, im)) if im.is_zero() => HNumber::from_real(quantize(&re, digits)),
        Some((re, im)) => HNumber::from_complex(quantize(&re, digits), quantize(&im, digits)),
        None => HNumber::nan(),
    }
}

fn unary_f64_real<F>(x: &HNumber, op: F, err: MathError) -> HNumber
where
    F: Fn(f64) -> f64,
{
    match x.real_value() {
        Some(v) => {
            let out = op(rational_to_f64(v));
            if out.is_nan() {
                HNumber::nan_with_error(err)
            } else {
                from_real_f64_approx(out)
            }
        }
        None => HNumber::nan_with_error(MathError::InvalidArg),
    }
}

fn from_real_f64_approx(v: f64) -> HNumber {
    if v.is_nan() {
        HNumber::nan()
    } else if v.is_infinite() {
        HNumber::nan_with_error(MathError::Overflow)
    } else {
        HNumber::from_real(from_f64_to_rational(v))
    }
}

fn from_f64_to_rational(v: f64) -> BigRational {
    parse_decimal_rational(&format!("{:.17}", v)).unwrap_or_else(|_| BigRational::zero())
}

fn is_e_constant(x: &HNumber) -> bool {
    x.real_value()
        .map(|v| quantize(v, 30) == quantize(e_r(), 30))
        .unwrap_or(false)
}

fn combinatoric(n: &HNumber, r: &HNumber, permutation: bool) -> HNumber {
    let (Some(nv), Some(rv)) = (integer_value(n), integer_value(r)) else {
        return HNumber::nan_with_error(MathError::InvalidArg);
    };
    if nv.is_negative() || rv.is_negative() || rv > nv {
        return HNumber::from_i64(0);
    }

    let mut r_eff = rv.clone();
    if !permutation {
        let alt = nv.clone() - rv.clone();
        if alt < r_eff {
            r_eff = alt;
        }
    }

    let Some(iterations) = r_eff.to_u64() else {
        return HNumber::nan_with_error(MathError::Overflow);
    };

    let mut result = BigInt::one();
    for i in 0..iterations {
        result *= nv.clone() - BigInt::from(i);
        if !permutation {
            result /= BigInt::from(i + 1);
        }
    }

    HNumber::from_bigint(result)
}

fn gamma_real(x: &BigRational) -> HNumber {
    if x.is_zero() || (x.is_negative() && x.is_integer()) {
        return HNumber::nan_with_error(MathError::OutOfDomain);
    }

    if x.is_integer() && x.is_positive() {
        let shifted = HNumber::from_real(x.clone() - BigRational::one());
        return HMath::factorial(&shifted);
    }

    let v = rational_to_f64(x);
    let g = 7.0;
    let c = [
        0.999_999_999_999_809_9,
        676.520_368_121_885_1,
        -1_259.139_216_722_402_8,
        771.323_428_777_653_1,
        -176.615_029_162_140_6,
        12.507_343_278_686_905,
        -0.138_571_095_265_720_12,
        9.984_369_578_019_572e-6,
        1.505_632_735_149_311_6e-7,
    ];

    let result = if v < 0.5 {
        std::f64::consts::PI / ((std::f64::consts::PI * v).sin() * gamma_lanczos(1.0 - v, g, &c))
    } else {
        gamma_lanczos(v, g, &c)
    };

    if result.is_finite() {
        from_real_f64_approx(result)
    } else {
        HNumber::nan_with_error(MathError::Overflow)
    }
}

fn gamma_lanczos(v: f64, g: f64, c: &[f64]) -> f64 {
    let x = v - 1.0;
    let mut a = c[0];
    for (i, coeff) in c.iter().enumerate().skip(1) {
        a += coeff / (x + i as f64);
    }
    let t = x + g + 0.5;
    (2.0 * std::f64::consts::PI).sqrt() * t.powf(x + 0.5) * (-t).exp() * a
}

fn erf_impl(x: f64) -> f64 {
    let a1 = 0.254829592;
    let a2 = -0.284496736;
    let a3 = 1.421413741;
    let a4 = -1.453152027;
    let a5 = 1.061405429;
    let p = 0.3275911;

    let sign = if x < 0.0 { -1.0 } else { 1.0 };
    let x = x.abs();
    let t = 1.0 / (1.0 + p * x);
    let y = 1.0 - (((((a5 * t + a4) * t) + a3) * t + a2) * t + a1) * t * (-x * x).exp();
    sign * y
}

fn pi_r() -> &'static BigRational {
    static VALUE: OnceLock<BigRational> = OnceLock::new();
    VALUE.get_or_init(|| {
        parse_decimal_rational("3.14159265358979323846264338327950288419716939937510582097494459230781640628620899862803482534211706798214808651328230664709384460955058223172535940813")
            .unwrap()
    })
}

fn e_r() -> &'static BigRational {
    static VALUE: OnceLock<BigRational> = OnceLock::new();
    VALUE.get_or_init(|| {
        parse_decimal_rational("2.71828182845904523536028747135266249775724709369995957496696762772407663035354759457138217852516642742746639193200305992181741359662904357290033429527")
            .unwrap()
    })
}

fn phi_r() -> &'static BigRational {
    static VALUE: OnceLock<BigRational> = OnceLock::new();
    VALUE.get_or_init(|| {
        parse_decimal_rational("1.61803398874989484820458683436563811772030917980576286213544862270526046281890244970720720418939113748475408807538689175212663386222353693179318006076")
            .unwrap()
    })
}

fn ln10_r() -> &'static BigRational {
    static VALUE: OnceLock<BigRational> = OnceLock::new();
    VALUE.get_or_init(|| {
        parse_decimal_rational("2.302585092994045684017991454684364207601101488628772976033327900967572609677352480235997205089598298341967784042286")
            .unwrap()
    })
}

fn ln2_r() -> &'static BigRational {
    static VALUE: OnceLock<BigRational> = OnceLock::new();
    VALUE.get_or_init(|| {
        parse_decimal_rational(
            "0.693147180559945309417232121458176568075500134360255254120680009493393621969",
        )
        .unwrap()
    })
}

fn two_pi_r() -> &'static BigRational {
    static VALUE: OnceLock<BigRational> = OnceLock::new();
    VALUE.get_or_init(|| pi_r().clone() * rat_from_i64(2))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_arithmetic() {
        let a = HNumber::from_f64(10.0);
        let b = HNumber::from_f64(3.0);
        assert_eq!((a.clone() + b.clone()).value(), 13.0);
        assert_eq!((a.clone() - b.clone()).value(), 7.0);
        assert_eq!((a.clone() * b.clone()).value(), 30.0);
        assert!((a.clone() / b.clone()).value() - 3.333333333 < 0.001);
        assert_eq!((a % b).value(), 1.0);
    }

    #[test]
    fn test_factorial() {
        assert_eq!(HMath::factorial(&HNumber::from_f64(5.0)).value(), 120.0);
        assert_eq!(HMath::factorial(&HNumber::from_f64(0.0)).value(), 1.0);
    }

    #[test]
    fn test_trig() {
        let pi = HMath::pi();
        assert!(HMath::sin(&pi).value().abs() < 1e-18);
        assert!((HMath::cos(&pi).value() + 1.0).abs() < 1e-18);
    }

    #[test]
    fn test_hex_parse() {
        let n = HNumber::from_str_radix("0xFF").unwrap();
        assert_eq!(n.value(), 255.0);
    }

    #[test]
    fn test_binary_parse() {
        let n = HNumber::from_str_radix("0b1010").unwrap();
        assert_eq!(n.value(), 10.0);
    }

    #[test]
    fn test_exact_decimal() {
        let a = HNumber::from_str_radix("0.1").unwrap();
        let b = HNumber::from_str_radix("0.2").unwrap();
        let c = HNumber::from_str_radix("0.3").unwrap();
        let result = a + b - c;
        assert_eq!(result.format_with(NumberFormat::General, -1, '.'), "0");
    }

    #[test]
    fn test_complex_display() {
        let z = HNumber::imaginary_unit();
        assert_eq!(z.to_string(), "i");
    }

    // ─── Number format parsing ───────────────────────────────────

    #[test]
    fn test_octal_parse() {
        let n = HNumber::from_str_radix("0o755").unwrap();
        assert_eq!(n.value(), 493.0);
    }

    #[test]
    fn test_hex_upper_parse() {
        let n = HNumber::from_str_radix("0X1A").unwrap();
        assert_eq!(n.value(), 26.0);
    }

    #[test]
    fn test_hash_hex_parse() {
        let n = HNumber::from_str_radix("#FF").unwrap();
        assert_eq!(n.value(), 255.0);
    }

    #[test]
    fn test_underscores_in_numbers() {
        let n = HNumber::from_str_radix("1_000_000").unwrap();
        assert_eq!(n.value(), 1_000_000.0);
    }

    // ─── Output format round-trips ──────────────────────────────

    #[test]
    fn test_format_hex_output() {
        let n = HNumber::from_f64(255.0);
        let s = n.format_with(NumberFormat::Hexadecimal, -1, '.');
        assert_eq!(s.to_lowercase(), "0xff");
    }

    #[test]
    fn test_format_octal_output() {
        let n = HNumber::from_f64(8.0);
        assert_eq!(n.format_with(NumberFormat::Octal, -1, '.'), "0o10");
    }

    #[test]
    fn test_format_binary_output() {
        let n = HNumber::from_f64(10.0);
        assert_eq!(n.format_with(NumberFormat::Binary, -1, '.'), "0b1010");
    }

    #[test]
    fn test_format_scientific() {
        let n = HNumber::from_f64(12345.0);
        let s = n.format_with(NumberFormat::Scientific, 4, '.');
        assert!(s.contains('e') || s.contains('E'));
    }

    #[test]
    fn test_format_fixed() {
        let n = HNumber::from_f64(3.14159);
        let s = n.format_with(NumberFormat::Fixed, 2, '.');
        assert_eq!(s, "3.14");
    }

    #[test]
    fn test_format_radix_comma() {
        let n = HNumber::from_f64(3.14);
        let s = n.format_with(NumberFormat::Fixed, 2, ',');
        assert_eq!(s, "3,14");
    }

    // ─── Complex arithmetic ─────────────────────────────────────

    #[test]
    fn test_complex_addition() {
        // i + i = 2i
        let i = HNumber::imaginary_unit();
        let r = i.clone() + i;
        let s = r.format_with(NumberFormat::General, -1, '.');
        assert_eq!(s, "2i");
    }

    #[test]
    fn test_complex_multiplication() {
        // i * i = -1
        let i = HNumber::imaginary_unit();
        let r = i.clone() * i;
        assert_eq!(r.value(), -1.0);
    }

    #[test]
    fn test_sqrt_negative() {
        // sqrt(-1) = i
        let n = HNumber::from_f64(-1.0);
        let r = HMath::sqrt(&n);
        assert_eq!(r.to_string(), "i");
    }

    // ─── Bitwise operations ─────────────────────────────────────

    #[test]
    fn test_bitwise_and() {
        let a = HNumber::from_f64(0b1100 as f64);
        let b = HNumber::from_f64(0b1010 as f64);
        assert_eq!((a & b).value(), 0b1000 as f64);
    }

    #[test]
    fn test_bitwise_or() {
        let a = HNumber::from_f64(0b1100 as f64);
        let b = HNumber::from_f64(0b1010 as f64);
        assert_eq!((a | b).value(), 0b1110 as f64);
    }

    #[test]
    fn test_bitwise_shift_left() {
        let a = HNumber::from_f64(1.0);
        let b = HNumber::from_f64(4.0);
        assert_eq!((a << b).value(), 16.0);
    }

    #[test]
    fn test_bitwise_shift_right() {
        let a = HNumber::from_f64(16.0);
        let b = HNumber::from_f64(2.0);
        assert_eq!((a >> b).value(), 4.0);
    }

    // ─── Special values ─────────────────────────────────────────

    #[test]
    fn test_pi_constant() {
        let pi = HMath::pi();
        assert!((pi.value() - std::f64::consts::PI).abs() < 1e-15);
    }

    #[test]
    fn test_e_constant() {
        let e = HMath::e();
        assert!((e.value() - std::f64::consts::E).abs() < 1e-15);
    }
}
