#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "openapi", schema(value_type = String))]
#[serde(transparent)]
pub struct Url(String);

impl<'de> serde::Deserialize<'de> for Url {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::new(s).map_err(serde::de::Error::custom)
    }
}

impl Url {
    pub fn new(value: impl Into<String>) -> Result<Self, UrlError> {
        let value = value.into().trim().to_string();
        if value.is_empty() {
            return Err(UrlError::Empty);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str { &self.0 }
}

impl std::fmt::Display for Url {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum UrlError {
    #[error("URL must not be empty")]
    Empty,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_url_succeeds() {
        let url = Url::new("https://example.com").unwrap();
        assert_eq!(url.as_str(), "https://example.com");
    }

    #[test]
    fn trims_whitespace() {
        let url = Url::new("  https://example.com  ").unwrap();
        assert_eq!(url.as_str(), "https://example.com");
    }

    #[test]
    fn empty_url_rejected() {
        assert!(matches!(Url::new("  "), Err(UrlError::Empty)));
    }

    #[test]
    fn display_works() {
        let url = Url::new("https://example.com").unwrap();
        assert_eq!(url.to_string(), "https://example.com");
    }
}
