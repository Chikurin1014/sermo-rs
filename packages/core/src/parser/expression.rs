use super::regex::ExtractionResult;
use crate::Result;
use std::fmt;

/// Represents a parsed expression
/// TODO: Extend with binary operations, function calls, etc.
#[derive(Debug, Clone)]
pub enum Expr {
    /// A reference to a capture group by index
    Ref(usize),
    /// A numeric value
    Value(f64),
    /// A string value
    Str(String),
}

impl Expr {
    /// Parse a string expression into an Expr
    /// Supports direct $n references and numeric/string literals.
    pub fn new(expr_str: &str) -> Self {
        let trimmed = expr_str.trim();

        // Try to parse as a capture reference like $1
        if let Some(idx) = parse_capture_reference(trimmed) {
            return Expr::Ref(idx);
        }

        // Try to parse as a numeric literal
        if let Ok(num) = trimmed.parse::<f64>() {
            return Expr::Value(num);
        }

        // Fall back to string
        Expr::Str(trimmed.to_string())
    }

    /// Evaluate the expression
    /// Currently returns self as-is since we don't have operations yet.
    /// TODO: Implement evaluation for binary operations, function calls, etc.
    pub fn eval(self, extraction: &ExtractionResult) -> Result<Self> {
        match self {
            Expr::Ref(idx) => {
                if let Ok(value) = get_capture_value(&extraction.captures, idx) {
                    return value
                        .parse::<f64>() // Try parse as a number
                        .map(Expr::Value)
                        .or_else(|_| Ok(Expr::Str(value))); // Fallback to string
                }
                Err(crate::Error::ParseError(format!(
                    "Cannot resolve capture reference ${}",
                    idx
                )))
            }
            other => Ok(other),
        }
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Ref(idx) => write!(f, "<ref: {}>", idx),
            Expr::Value(num) => write!(f, "{}", num),
            Expr::Str(s) => write!(f, "{}", s),
        }
    }
}

impl TryFrom<&Expr> for f64 {
    type Error = crate::Error;

    fn try_from(expr: &Expr) -> Result<Self> {
        match expr {
            Expr::Value(num) => Ok(*num),
            Expr::Ref(_) => Err(crate::Error::ParseError(
                "Cannot convert reference to f64 directly".to_string(),
            )),
            Expr::Str(s) => s
                .parse::<f64>()
                .map_err(|_| crate::Error::ParseError(format!("Cannot convert '{}' to f64", s))),
        }
    }
}

impl TryFrom<Expr> for f64 {
    type Error = crate::Error;

    fn try_from(expr: Expr) -> Result<Self> {
        (&expr).try_into()
    }
}

impl TryFrom<&Expr> for String {
    type Error = crate::Error;

    fn try_from(expr: &Expr) -> Result<Self> {
        if let Expr::Ref(_) = expr {
            return Err(crate::Error::ParseError(
                "Cannot convert reference to String directly".to_string(),
            ));
        }
        Ok(expr.to_string())
    }
}

impl TryFrom<Expr> for String {
    type Error = crate::Error;

    fn try_from(expr: Expr) -> Result<Self> {
        (&expr).try_into()
    }
}

/// Parse a simple capture reference like "$1" or "$5"
fn parse_capture_reference(s: &str) -> Option<usize> {
    if s.starts_with('$') && s.len() > 1 {
        s[1..].parse::<usize>().ok()
    } else {
        None
    }
}

/// Get a capture value by index
fn get_capture_value(captures: &[String], index: usize) -> Result<String> {
    captures
        .get(index)
        .cloned()
        .ok_or_else(|| crate::Error::ParseError(format!("Capture group ${} not found", index)))
}

// TODO: Implement arithmetic expression evaluator (+, -, *, /)

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_capture_reference() {
        let extraction = ExtractionResult {
            captures: vec!["full".to_string(), "42.5".to_string()],
        };
        let expr = Expr::new("$1").eval(&extraction).unwrap();
        match expr {
            Expr::Value(num) => assert_eq!(num, 42.5),
            _ => panic!("Expected Value"),
        }
    }

    #[test]
    fn test_parse_string_reference() {
        let extraction = ExtractionResult {
            captures: vec!["full".to_string(), "hello".to_string()],
        };
        let expr = Expr::new("$1").eval(&extraction).unwrap();
        match expr {
            Expr::Str(s) => assert_eq!(s, "hello"),
            _ => panic!("Expected Str"),
        }
    }

    #[test]
    fn test_parse_numeric_literal() {
        let extraction = ExtractionResult { captures: vec![] };
        let expr = Expr::new("42.5").eval(&extraction).unwrap();
        match expr {
            Expr::Value(num) => assert_eq!(num, 42.5),
            _ => panic!("Expected Value"),
        }
    }

    #[test]
    fn test_parse_string_literal() {
        let extraction = ExtractionResult { captures: vec![] };
        let expr = Expr::new("hello").eval(&extraction).unwrap();
        match expr {
            Expr::Str(s) => assert_eq!(s, "hello"),
            _ => panic!("Expected Str"),
        }
    }

    #[test]
    fn test_eval_f64() {
        let expr = Expr::Value(42.5);
        let num: f64 = (&expr).try_into().unwrap();
        assert_eq!(num, 42.5);
    }

    #[test]
    fn test_eval_string() {
        let expr = Expr::Value(42.5);
        assert_eq!(expr.to_string(), "42.5");
    }

    #[test]
    fn test_display_string_expr() {
        let expr = Expr::Str("hello".to_string());
        assert_eq!(expr.to_string(), "hello");
    }

    #[test]
    fn test_invalid_capture_reference() {
        let extraction = ExtractionResult {
            captures: vec!["full".to_string(), "42".to_string()],
        };
        let result = Expr::new("$5").eval(&extraction);
        assert!(result.is_err());
    }
}
