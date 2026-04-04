use meerkat_application::ports::fingerprint_service::FingerprintService;
use meerkat_domain::models::issue::FingerprintHash;
use sha2::{Digest, Sha256};

pub struct Sha256FingerprintService;

impl FingerprintService for Sha256FingerprintService {
    fn compute(&self, exception_type: Option<String>, exception_value: Option<String>, message: String) -> FingerprintHash {
        let input = match (exception_type.as_deref(), exception_value.as_deref()) {
            (Some(t), Some(v)) => format!("{t}:{v}"),
            (Some(t), None) => format!("{t}:"),
            _ => message,
        };

        let hash = Sha256::digest(input.as_bytes());
        FingerprintHash::new(hex::encode(hash)).expect("SHA-256 hash is never empty")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn svc() -> Sha256FingerprintService {
        Sha256FingerprintService
    }

    #[test]
    fn given_same_exception_then_produces_same_hash() {
        // act
        let hash1 = svc().compute(Some("TypeError".into()), Some("x is not defined".into()), "msg".into());
        let hash2 = svc().compute(Some("TypeError".into()), Some("x is not defined".into()), "msg".into());

        // assert
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn given_different_exception_type_then_produces_different_hash() {
        // act
        let hash1 = svc().compute(Some("TypeError".into()), Some("x is not defined".into()), "msg".into());
        let hash2 = svc().compute(Some("ValueError".into()), Some("x is not defined".into()), "msg".into());

        // assert
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn given_no_exception_then_falls_back_to_message() {
        // act
        let hash1 = svc().compute(None, None, "something broke".into());
        let hash2 = svc().compute(None, None, "something broke".into());

        // assert
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn given_no_exception_and_different_messages_then_produces_different_hash() {
        // act
        let hash1 = svc().compute(None, None, "something broke".into());
        let hash2 = svc().compute(None, None, "something else broke".into());

        // assert
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn given_valid_input_then_returns_hex_encoded_sha256() {
        // act
        let hash = svc().compute(Some("TypeError".into()), Some("x".into()), "msg".into());

        // assert
        assert_eq!(hash.as_str().len(), 64);
        assert!(hash.as_str().chars().all(|c| c.is_ascii_hexdigit()));
    }
}
