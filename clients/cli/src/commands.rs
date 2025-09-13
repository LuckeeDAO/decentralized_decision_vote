use serde_json::json;
use shared_types::*;
use crate::client::{ApiClient, ApiError};
use tracing::{error};

/// Create a new vote
pub async fn create_vote(
    client: &ApiClient,
    title: String,
    description: String,
    template: String,
    params: Option<String>,
    commitment_hours: u32,
    reveal_hours: u32,
) -> Result<(), ApiError> {
    let template_params = if let Some(params_str) = params {
        serde_json::from_str(&params_str)?
    } else {
        json!({})
    };
    
    let config = VoteConfig {
        title,
        description,
        template_id: template,
        template_params,
        commitment_duration_hours: commitment_hours,
        reveal_duration_hours: reveal_hours,
    };
    
    match client.create_vote(config).await {
        Ok(response) => {
            println!("✅ Vote created successfully!");
            println!("Vote ID: {}", response.vote_id);
            println!("Message: {}", response.message);
        }
        Err(e) => {
            error!("Failed to create vote: {}", e);
            return Err(e);
        }
    }
    
    Ok(())
}

/// Get vote information
pub async fn get_vote(client: &ApiClient, vote_id: String) -> Result<(), ApiError> {
    match client.get_vote(&vote_id).await {
        Ok(response) => {
            println!("📊 Vote Information");
            println!("==================");
            println!("ID: {}", response.vote.id);
            println!("Title: {}", response.vote.title);
            println!("Description: {}", response.vote.description);
            println!("Template: {}", response.vote.template_id);
            println!("Creator: {}", response.vote.creator);
            println!("Status: {:?}", response.vote.status);
            println!("Created: {}", response.vote.created_at);
            println!("Commitment Phase: {} - {}", response.vote.commitment_start, response.vote.commitment_end);
            println!("Reveal Phase: {} - {}", response.vote.reveal_start, response.vote.reveal_end);
            
            if let Some(results) = response.vote.results {
                println!("Results: {}", serde_json::to_string_pretty(&results).unwrap_or_default());
            }
        }
        Err(e) => {
            error!("Failed to get vote: {}", e);
            return Err(e);
        }
    }
    
    Ok(())
}

/// List votes
pub async fn list_votes(
    client: &ApiClient,
    page: u32,
    size: u32,
    status: Option<String>,
    creator: Option<String>,
) -> Result<(), ApiError> {
    let status_filter = if let Some(status_str) = status {
        Some(match status_str.as_str() {
            "created" => VoteStatus::Created,
            "commitment_phase" => VoteStatus::CommitmentPhase,
            "reveal_phase" => VoteStatus::RevealPhase,
            "completed" => VoteStatus::Completed,
            "cancelled" => VoteStatus::Cancelled,
            _ => {
                error!("Invalid status: {}", status_str);
                return Err(ApiError::ApiError {
                    message: format!("Invalid status: {}", status_str),
                });
            }
        })
    } else {
        None
    };
    
    let query = ListQuery {
        page,
        page_size: size,
        status: status_filter,
        creator,
    };
    
    match client.list_votes(query).await {
        Ok(response) => {
            println!("📋 Votes (Page {}/{} - {} total)", 
                response.votes.page + 1, 
                response.votes.total_pages, 
                response.votes.total
            );
            println!("==================");
            
            for vote in response.votes.items {
                println!("• {} - {} ({:?})", vote.id, vote.title, vote.status);
                println!("  Created: {} by {}", vote.created_at, vote.creator);
                println!();
            }
        }
        Err(e) => {
            error!("Failed to list votes: {}", e);
            return Err(e);
        }
    }
    
    Ok(())
}

/// Submit a commitment
pub async fn commit_vote(
    client: &ApiClient,
    vote_id: String,
    voter: String,
    value: String,
    salt: Option<String>,
) -> Result<(), ApiError> {
    let (commitment_hash, salt) = client.create_commitment(&value, salt);
    
    let request = CommitRequest {
        voter,
        commitment_hash,
        salt,
    };
    
    match client.commit_vote(&vote_id, request).await {
        Ok(response) => {
            println!("✅ Commitment submitted successfully!");
            println!("Commitment ID: {}", response.commitment_id);
            println!("Message: {}", response.message);
        }
        Err(e) => {
            error!("Failed to submit commitment: {}", e);
            return Err(e);
        }
    }
    
    Ok(())
}

/// Submit a reveal
pub async fn reveal_vote(
    client: &ApiClient,
    vote_id: String,
    voter: String,
    value: String,
    salt: String,
) -> Result<(), ApiError> {
    let parsed_value = serde_json::from_str(&value)
        .map_err(|e| ApiError::JsonError(e))?;
    
    let request = RevealRequest {
        voter,
        value: parsed_value,
        salt,
    };
    
    match client.reveal_vote(&vote_id, request).await {
        Ok(response) => {
            println!("✅ Reveal submitted successfully!");
            println!("Reveal ID: {}", response.reveal_id);
            println!("Message: {}", response.message);
        }
        Err(e) => {
            error!("Failed to submit reveal: {}", e);
            return Err(e);
        }
    }
    
    Ok(())
}

/// Get vote results
pub async fn get_results(client: &ApiClient, vote_id: String) -> Result<(), ApiError> {
    match client.get_results(&vote_id).await {
        Ok(response) => {
            println!("📊 Vote Results");
            println!("===============");
            println!("Vote ID: {}", response.results.vote_id);
            println!("Total Votes: {}", response.results.total_votes);
            println!("Calculated: {}", response.results.calculated_at);
            println!("Results:");
            println!("{}", serde_json::to_string_pretty(&response.results.results).unwrap_or_default());
        }
        Err(e) => {
            error!("Failed to get results: {}", e);
            return Err(e);
        }
    }
    
    Ok(())
}

/// List available templates
pub async fn list_templates(client: &ApiClient) -> Result<(), ApiError> {
    match client.list_templates().await {
        Ok(response) => {
            println!("📋 Available Templates");
            println!("=====================");
            
            if let Some(templates) = response.get("templates").and_then(|t| t.as_array()) {
                for template in templates {
                    if let Some(template_str) = template.as_str() {
                        println!("• {}", template_str);
                    }
                }
            }
        }
        Err(e) => {
            error!("Failed to list templates: {}", e);
            return Err(e);
        }
    }
    
    Ok(())
}

/// Get template information
pub async fn get_template(client: &ApiClient, template_id: String) -> Result<(), ApiError> {
    match client.get_template(&template_id).await {
        Ok(response) => {
            println!("📋 Template Information");
            println!("======================");
            println!("{}", serde_json::to_string_pretty(&response).unwrap_or_default());
        }
        Err(e) => {
            error!("Failed to get template: {}", e);
            return Err(e);
        }
    }
    
    Ok(())
}

/// Verify vote results
pub async fn verify_results(client: &ApiClient, vote_id: String) -> Result<(), ApiError> {
    match client.verify_results(&vote_id).await {
        Ok(response) => {
            println!("🔍 Vote Results Verification");
            println!("============================");
            println!("Vote ID: {}", response.verification.vote_id);
            println!("Valid: {}", if response.verification.is_valid { "✅ YES" } else { "❌ NO" });
            println!("Verification Time: {}", response.verification.verification_timestamp);
            
            println!("\n📋 Commitment Verification:");
            println!("  Total Commitments: {}", response.verification.commitment_verification.total_commitments);
            println!("  Verified: {}", response.verification.commitment_verification.verified_commitments);
            println!("  Failed: {}", response.verification.commitment_verification.failed_commitments);
            
            if !response.verification.commitment_verification.commitment_issues.is_empty() {
                println!("  Issues:");
                for issue in &response.verification.commitment_verification.commitment_issues {
                    println!("    • {}", issue);
                }
            }
            
            println!("\n📊 Results Verification:");
            println!("  Total Reveals: {}", response.verification.results_verification.total_reveals);
            println!("  Valid Reveals: {}", response.verification.results_verification.valid_reveals);
            println!("  Invalid Reveals: {}", response.verification.results_verification.invalid_reveals);
            println!("  Random Seed Valid: {}", if response.verification.results_verification.random_seed_verification { "✅" } else { "❌" });
            println!("  Algorithm Valid: {}", if response.verification.results_verification.selection_algorithm_verification { "✅" } else { "❌" });
            
            if !response.verification.results_verification.results_issues.is_empty() {
                println!("  Issues:");
                for issue in &response.verification.results_verification.results_issues {
                    println!("    • {}", issue);
                }
            }
            
            if !response.verification.issues.is_empty() {
                println!("\n⚠️  Overall Issues:");
                for issue in &response.verification.issues {
                    println!("  • {}", issue);
                }
            }
        }
        Err(e) => {
            error!("Failed to verify results: {}", e);
            return Err(e);
        }
    }
    
    Ok(())
}

/// Health check
pub async fn health_check(client: &ApiClient) -> Result<(), ApiError> {
    match client.health_check().await {
        Ok(response) => {
            println!("🏥 Health Check");
            println!("===============");
            println!("Status: {}", response.status);
            println!("Version: {}", response.version);
            println!("Timestamp: {}", response.timestamp);
            println!("Services:");
            
            for (name, status) in response.services {
                println!("  • {}: {}", name, status.status);
                if let Some(message) = status.message {
                    println!("    {}", message);
                }
            }
        }
        Err(e) => {
            error!("Health check failed: {}", e);
            return Err(e);
        }
    }
    
    Ok(())
}
