#[derive(Debug, Clone)]
pub struct SearchFilter {
    raw: String,
}

impl SearchFilter {
    pub fn new(input: &str) -> Option<Self> {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return None;
        }
        Some(Self {
            raw: trimmed.to_string(),
        })
    }

    pub fn contains_pattern(&self) -> String {
        format!("%{}%", self.raw)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_empty_input_then_returns_none() {
        assert!(SearchFilter::new("").is_none());
        assert!(SearchFilter::new("   ").is_none());
    }

    #[test]
    fn given_valid_input_then_returns_contains_pattern() {
        let filter = SearchFilter::new("  foo  ").unwrap();
        assert_eq!(filter.contains_pattern(), "%foo%");
    }
}
