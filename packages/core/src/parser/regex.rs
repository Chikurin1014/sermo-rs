use crate::Result;
use regex::Regex;

/// Result of extraction from a string
#[derive(Debug, Clone)]
pub struct ExtractionResult {
    /// Captured groups from the regex match
    pub captures: Vec<String>,
}

/// Extract captures from input string using a regex pattern
pub fn extract(pattern: &str, input: &str) -> Result<ExtractionResult> {
    let regex = Regex::new(pattern)
        .map_err(|e| crate::Error::ParseError(format!("Invalid regex pattern: {}", e)))?;

    let captures = regex
        .captures(input)
        .ok_or_else(|| crate::Error::ParseError("No match found".to_string()))?;

    let captures_vec = captures
        .iter()
        .map(|m| m.map_or_else(String::new, |m| m.as_str().to_string()))
        .collect::<Vec<String>>();

    Ok(ExtractionResult {
        captures: captures_vec,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_regex_extraction() {
        let result = extract(r"Temperature: (\d+\.\d+)", "Temperature: 25.5").unwrap();
        assert_eq!(result.captures[1], "25.5");
    }

    #[test]
    fn test_multiple_captures() {
        let result = extract(r"(\w+): (\d+\.\d+)", "Temperature: 25.5").unwrap();
        assert_eq!(result.captures[1], "Temperature");
        assert_eq!(result.captures[2], "25.5");
    }

    #[test]
    fn test_no_match() {
        let result = extract(r"Temperature: (\d+\.\d+)", "Humidity: 50%");
        assert!(result.is_err());
    }
}
