use sha2::{Sha256, Digest};
use hex;
use uuid::Uuid;

/// Generate a random salt for commitment schemes
pub fn generate_salt() -> String {
    Uuid::new_v4().to_string()
}

/// Hash a value with SHA256
pub fn hash_value(value: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(value.as_bytes());
    hex::encode(hasher.finalize())
}

/// Create a commitment hash from value and salt
pub fn create_commitment(value: &str, salt: &str) -> String {
    let combined = format!("{}:{}", value, salt);
    hash_value(&combined)
}

/// Verify a commitment by checking if the hash matches
pub fn verify_commitment(value: &str, salt: &str, expected_hash: &str) -> bool {
    let actual_hash = create_commitment(value, salt);
    actual_hash == expected_hash
}

/// Generate a random UUID
pub fn generate_id() -> String {
    Uuid::new_v4().to_string()
}

/// Hash a string with a given algorithm
pub fn hash_with_algorithm(data: &str, algorithm: &str) -> String {
    match algorithm.to_lowercase().as_str() {
        "sha256" => {
            let mut hasher = Sha256::new();
            hasher.update(data.as_bytes());
            hex::encode(hasher.finalize())
        }
        _ => {
            // Default to SHA256
            hash_value(data)
        }
    }
}
