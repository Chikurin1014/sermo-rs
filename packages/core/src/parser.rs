pub mod expression;
pub mod regex;

use crate::data::{Message, Point};
use crate::Result;

use expression::Expr;
use regex::extract;

/// Regex-based parser using extraction and expression evaluation
pub struct Parser {
    pattern_str: String,
    label_expr_str: String,
    value_expr_str: String,
}

impl Parser {
    pub fn new(
        pattern: impl Into<String>,
        name_expr: impl Into<String>,
        value_expr: impl Into<String>,
    ) -> Self {
        Self {
            pattern_str: pattern.into(),
            label_expr_str: name_expr.into(),
            value_expr_str: value_expr.into(),
        }
    }

    pub fn parse(&self, message: &Message) -> Result<Point> {
        let extraction = extract(&self.pattern_str, &message.data)?;
        let value_expr = Expr::new(&self.value_expr_str).eval(&extraction)?;
        let value: f64 = value_expr.try_into()?;
        let mut point = Point::new(value).with_timestamp(message.timestamp);

        // If label_expr is provided, evaluate it
        if !self.label_expr_str.is_empty() {
            let label_expr = Expr::new(&self.label_expr_str).eval(&extraction)?;
            let label: String = label_expr.try_into()?;
            point = point.with_label(label);
        }

        Ok(point)
    }

    pub fn is_complete(&self, line: &str) -> bool {
        !line.trim().is_empty()
    }

    pub fn pattern(&self) -> &str {
        &self.pattern_str
    }

    pub fn label_expr(&self) -> &str {
        &self.label_expr_str
    }

    pub fn value_expr(&self) -> &str {
        &self.value_expr_str
    }

    pub fn set_pattern(&mut self, pattern: impl Into<String>) {
        self.pattern_str = pattern.into();
    }

    pub fn set_label_expr(&mut self, label_expr: impl Into<String>) {
        self.label_expr_str = label_expr.into();
    }

    pub fn set_value_expr(&mut self, value_expr: impl Into<String>) {
        self.value_expr_str = value_expr.into();
    }
}

#[cfg(test)]
mod tests {
    use crate::data::Direction;

    use super::*;

    #[test]
    fn test_regex_parser_simple_numeric() {
        let parser = Parser::new(r"(\d+\.\d+)", "", "$1");
        let point = parser
            .parse(&Message {
                timestamp: 0,
                direction: Direction::In,
                data: "Value: 42.5".to_string(),
            })
            .unwrap();
        assert_eq!(point.value, 42.5);
        assert_eq!(point.label, None);
    }

    #[test]
    fn test_regex_parser_with_name() {
        let parser = Parser::new(r"(\w+): (\d+\.\d+)", "$1", "$2");
        let point = parser
            .parse(&Message {
                timestamp: 0,
                direction: Direction::In,
                data: "Temperature: 25.5".to_string(),
            })
            .unwrap();
        assert_eq!(point.value, 25.5);
        assert_eq!(point.label, Some("Temperature".to_string()));
    }

    #[test]
    fn test_regex_parser_multiple_captures() {
        let parser = Parser::new(r"(\w+)=(\d+)", "$1", "$2");
        let point = parser
            .parse(&Message {
                timestamp: 0,
                direction: Direction::In,
                data: "Humidity=65".to_string(),
            })
            .unwrap();
        assert_eq!(point.value, 65.0);
        assert_eq!(point.label, Some("Humidity".to_string()));
    }

    #[test]
    fn test_regex_parser_no_match() {
        let parser = Parser::new(r"Temperature: (\d+\.\d+)", "", "$1");
        let result = parser.parse(&Message {
            timestamp: 0,
            direction: Direction::In,
            data: "Humidity: 50%".to_string(),
        });
        assert!(result.is_err());
    }

    #[test]
    fn test_regex_parser_invalid_number() {
        let parser = Parser::new(r"(\w+)", "", "$1");
        let result = parser.parse(&Message {
            timestamp: 0,
            direction: Direction::In,
            data: "Value: not_a_number".to_string(),
        });
        assert!(result.is_err());
    }
}
