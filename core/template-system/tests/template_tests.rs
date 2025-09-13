use template_system::*;

#[tokio::test]
async fn test_template_registry_register_and_get() {
    let mut registry = TemplateRegistry::new();
    
    // Create a simple test template
    let template = YesNoTemplate::new();
    let template_id = template.id().to_string();
    
    // Register the template
    registry.register(template);
    
    // Verify it was registered
    assert!(registry.exists(&template_id));
    assert_eq!(registry.count(), 1);
    
    // Get the template
    let retrieved = registry.get(&template_id);
    assert!(retrieved.is_ok());
    
    let template = retrieved.unwrap();
    assert_eq!(template.id(), template_id);
}

#[tokio::test]
async fn test_template_registry_get_nonexistent() {
    let registry = TemplateRegistry::new();
    
    let result = registry.get("nonexistent");
    assert!(result.is_err());
    
    match result.unwrap_err() {
        TemplateError::TemplateNotFound { id } => {
            assert_eq!(id, "nonexistent");
        }
        _ => panic!("Expected TemplateNotFound error"),
    }
}

#[tokio::test]
async fn test_template_registry_list() {
    let mut registry = TemplateRegistry::new();
    
    // Register multiple templates
    registry.register(YesNoTemplate::new());
    registry.register(MultipleChoiceTemplate::new());
    
    let templates = registry.list();
    assert_eq!(templates.len(), 2);
    assert!(templates.contains(&"yes_no".to_string()));
    assert!(templates.contains(&"multiple_choice".to_string()));
}

#[tokio::test]
async fn test_default_template_registry() {
    let registry = DefaultTemplateRegistry::new();
    
    // Should have built-in templates
    assert!(registry.count() >= 4);
    assert!(registry.exists("yes_no"));
    assert!(registry.exists("multiple_choice"));
    assert!(registry.exists("numeric_range"));
    assert!(registry.exists("ranking"));
}

#[tokio::test]
async fn test_yes_no_template_validation() {
    let template = YesNoTemplate::new();
    let params = serde_json::json!({});
    
    // Valid values
    assert!(template.validate(&serde_json::json!(true), &params).await.is_ok());
    assert!(template.validate(&serde_json::json!(false), &params).await.is_ok());
    
    // Invalid values
    assert!(template.validate(&serde_json::json!("maybe"), &params).await.is_err());
    assert!(template.validate(&serde_json::json!(""), &params).await.is_err());
    assert!(template.validate(&serde_json::json!(123), &params).await.is_err());
}

#[tokio::test]
async fn test_yes_no_template_canonicalization() {
    let template = YesNoTemplate::new();
    let params = serde_json::json!({});
    
    let canonical = template.canonicalize(&serde_json::json!(true), &params).await.unwrap();
    assert_eq!(canonical, b"yes");
    
    let canonical = template.canonicalize(&serde_json::json!(false), &params).await.unwrap();
    assert_eq!(canonical, b"no");
}

#[tokio::test]
async fn test_yes_no_template_aggregation() {
    let template = YesNoTemplate::new();
    let params = serde_json::json!({});
    
    let values = vec![serde_json::json!(true), serde_json::json!(false), serde_json::json!(true), serde_json::json!(true), serde_json::json!(false)];
    let result = template.aggregate(&values, &params).await.unwrap();
    
    // Should have counts for yes and no
    assert_eq!(result["yes"], serde_json::json!(3));
    assert_eq!(result["no"], serde_json::json!(2));
    assert_eq!(result["total"], serde_json::json!(5));
}

#[tokio::test]
async fn test_multiple_choice_template_validation() {
    let template = MultipleChoiceTemplate::new();
    
    // Set up options
    let params = serde_json::json!({"choices": ["option1", "option2", "option3"]});
    
    // Valid values
    assert!(template.validate(&serde_json::json!("option1"), &params).await.is_ok());
    assert!(template.validate(&serde_json::json!("option2"), &params).await.is_ok());
    assert!(template.validate(&serde_json::json!("option3"), &params).await.is_ok());
    
    // Invalid values
    assert!(template.validate(&serde_json::json!("option4"), &params).await.is_err());
    assert!(template.validate(&serde_json::json!(""), &params).await.is_err());
}

#[tokio::test]
async fn test_multiple_choice_template_aggregation() {
    let template = MultipleChoiceTemplate::new();
    
    // Set up options
    let params = serde_json::json!({"choices": ["A", "B", "C"]});
    
    let values = vec![serde_json::json!("A"), serde_json::json!("B"), serde_json::json!("A"), serde_json::json!("C"), serde_json::json!("B"), serde_json::json!("A")];
    let result = template.aggregate(&values, &params).await.unwrap();
    
    assert_eq!(result["total"], serde_json::json!(6));
    assert_eq!(result["results"]["A"], serde_json::json!(3));
    assert_eq!(result["results"]["B"], serde_json::json!(2));
    assert_eq!(result["results"]["C"], serde_json::json!(1));
}

#[tokio::test]
async fn test_numeric_range_template_validation() {
    let template = NumericRangeTemplate::new();
    
    // Set up range
    let params = serde_json::json!({"min": 1, "max": 10});
    
    // Valid values
    assert!(template.validate(&serde_json::json!(5), &params).await.is_ok());
    assert!(template.validate(&serde_json::json!(1), &params).await.is_ok());
    assert!(template.validate(&serde_json::json!(10), &params).await.is_ok());
    
    // Invalid values
    assert!(template.validate(&serde_json::json!(0), &params).await.is_err());
    assert!(template.validate(&serde_json::json!(11), &params).await.is_err());
    assert!(template.validate(&serde_json::json!("abc"), &params).await.is_err());
    assert!(template.validate(&serde_json::json!(""), &params).await.is_err());
}

#[tokio::test]
async fn test_numeric_range_template_aggregation() {
    let template = NumericRangeTemplate::new();
    
    // Set up range
    let params = serde_json::json!({"min": 1, "max": 5});
    
    let values = vec![serde_json::json!(1), serde_json::json!(2), serde_json::json!(3), serde_json::json!(2), serde_json::json!(4), serde_json::json!(1), serde_json::json!(5)];
    let result = template.aggregate(&values, &params).await.unwrap();
    
    assert_eq!(result["count"], serde_json::json!(7));
    assert_eq!(result["sum"], serde_json::json!(18.0));
    assert_eq!(result["average"], serde_json::json!(18.0 / 7.0));
    assert_eq!(result["min"], serde_json::json!(1.0));
    assert_eq!(result["max"], serde_json::json!(5.0));
}

#[tokio::test]
async fn test_ranking_template_validation() {
    let template = RankingTemplate::new();
    
    // Set up items to rank
    let params = serde_json::json!({"options": ["item1", "item2", "item3"]});
    
    // Valid ranking
    let valid_ranking = serde_json::json!(["item1", "item2", "item3"]);
    assert!(template.validate(&valid_ranking, &params).await.is_ok());
    
    // Invalid ranking (missing item)
    let invalid_ranking = serde_json::json!(["item1", "item2"]);
    assert!(template.validate(&invalid_ranking, &params).await.is_err());
    
    // Invalid ranking (extra item)
    let invalid_ranking = serde_json::json!(["item1", "item2", "item3", "item4"]);
    assert!(template.validate(&invalid_ranking, &params).await.is_err());
}

#[tokio::test]
async fn test_ranking_template_aggregation() {
    let template = RankingTemplate::new();
    
    // Set up items to rank
    let params = serde_json::json!({"options": ["A", "B", "C"]});
    
    let rankings = vec![
        serde_json::json!(["A", "B", "C"]),
        serde_json::json!(["B", "A", "C"]),
        serde_json::json!(["A", "C", "B"]),
    ];
    
    let result = template.aggregate(&rankings, &params).await.unwrap();
    
    // Should have aggregated ranking results
    assert!(result["ranking"].is_array());
    let ranking_array = result["ranking"].as_array().unwrap();
    assert_eq!(ranking_array.len(), 3);
    
    // Check that all options are present in the ranking
    let options: Vec<String> = ranking_array.iter()
        .map(|item| item["option"].as_str().unwrap().to_string())
        .collect();
    assert!(options.contains(&"A".to_string()));
    assert!(options.contains(&"B".to_string()));
    assert!(options.contains(&"C".to_string()));
}
