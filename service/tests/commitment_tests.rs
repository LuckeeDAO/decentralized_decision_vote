use decentralized_decision_vote::core::template::{BitTemplate, OptionIndexTemplate, StringTemplate, VoteValueTemplate};
use serde_json::json;

#[test]
fn test_bit_template_validation() {
    let template = BitTemplate;
    
    // Valid cases
    assert!(template.validate(&json!(true), &json!({})).is_ok());
    assert!(template.validate(&json!(false), &json!({})).is_ok());
    assert!(template.validate(&json!(1), &json!({})).is_ok());
    assert!(template.validate(&json!(0), &json!({})).is_ok());
    
    // Invalid cases
    assert!(template.validate(&json!(2), &json!({})).is_err());
    assert!(template.validate(&json!("true"), &json!({})).is_err());
    assert!(template.validate(&json!(null), &json!({})).is_err());
}

#[test]
fn test_bit_template_canonicalize() {
    let template = BitTemplate;
    
    assert_eq!(template.canonicalize(&json!(true), &json!({})).unwrap(), vec![1]);
    assert_eq!(template.canonicalize(&json!(false), &json!({})).unwrap(), vec![0]);
    assert_eq!(template.canonicalize(&json!(1), &json!({})).unwrap(), vec![1]);
    assert_eq!(template.canonicalize(&json!(0), &json!({})).unwrap(), vec![0]);
}

#[test]
fn test_option_index_template_validation() {
    let template = OptionIndexTemplate;
    let params = json!({"max": 3});
    
    // Valid cases
    assert!(template.validate(&json!(0), &params).is_ok());
    assert!(template.validate(&json!(1), &params).is_ok());
    assert!(template.validate(&json!(2), &params).is_ok());
    
    // Invalid cases
    assert!(template.validate(&json!(3), &params).is_err());
    assert!(template.validate(&json!(4), &params).is_err());
    assert!(template.validate(&json!(-1), &params).is_err());
    assert!(template.validate(&json!("0"), &params).is_err());
}

#[test]
fn test_option_index_template_canonicalize() {
    let template = OptionIndexTemplate;
    let params = json!({"max": 3});
    
    assert_eq!(template.canonicalize(&json!(0), &params).unwrap(), vec![0, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(template.canonicalize(&json!(1), &params).unwrap(), vec![0, 0, 0, 0, 0, 0, 0, 1]);
    assert_eq!(template.canonicalize(&json!(2), &params).unwrap(), vec![0, 0, 0, 0, 0, 0, 0, 2]);
}

#[test]
fn test_string_template_validation() {
    let template = StringTemplate;
    let params = json!({"max_len": 10});
    
    // Valid cases
    assert!(template.validate(&json!("hello"), &params).is_ok());
    assert!(template.validate(&json!(""), &json!({})).is_ok());
    
    // Invalid cases
    assert!(template.validate(&json!("very long string"), &params).is_err());
    assert!(template.validate(&json!(123), &json!({})).is_err());
}

#[test]
fn test_string_template_canonicalize() {
    let template = StringTemplate;
    
    assert_eq!(template.canonicalize(&json!("hello"), &json!({})).unwrap(), b"hello".to_vec());
    assert_eq!(template.canonicalize(&json!(""), &json!({})).unwrap(), b"".to_vec());
}

#[test]
fn test_commitment_algorithm_consistency() {
    use sha2::{Sha256, Digest};
    use hex::ToHex;
    
    // Test that same value + salt produces same commitment
    let value = b"test_value";
    let salt = b"test_salt";
    
    let mut hasher1 = Sha256::new();
    hasher1.update(b"commit|");
    hasher1.update(value);
    hasher1.update(b"|");
    hasher1.update(salt);
    let commitment1: String = hasher1.finalize().encode_hex();
    
    let mut hasher2 = Sha256::new();
    hasher2.update(b"commit|");
    hasher2.update(value);
    hasher2.update(b"|");
    hasher2.update(salt);
    let commitment2: String = hasher2.finalize().encode_hex();
    
    assert_eq!(commitment1, commitment2);
    
    // Test that different salt produces different commitment
    let mut hasher3 = Sha256::new();
    hasher3.update(b"commit|");
    hasher3.update(value);
    hasher3.update(b"|");
    hasher3.update(b"different_salt");
    let commitment3: String = hasher3.finalize().encode_hex();
    
    assert_ne!(commitment1, commitment3);
}
