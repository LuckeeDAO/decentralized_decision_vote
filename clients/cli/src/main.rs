use clap::{Parser, Subcommand};
use shared_logging::init_logging_from_env;
use tracing::info;

mod commands;
mod client;
mod config;

use commands::*;

#[derive(Parser)]
#[command(name = "vote")]
#[command(about = "A command-line interface for the decentralized decision vote system")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    
    /// API server URL
    #[arg(long, default_value = "http://localhost:8080")]
    api_url: String,
    
    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new vote
    Create {
        /// Vote title
        #[arg(short = 't', long)]
        title: String,
        
        /// Vote description
        #[arg(short, long)]
        description: String,
        
        /// Template ID
        #[arg(short = 'T', long)]
        template: String,
        
        /// Template parameters (JSON)
        #[arg(short, long)]
        params: Option<String>,
        
        /// Commitment duration in hours
        #[arg(long, default_value = "24")]
        commitment_hours: u32,
        
        /// Reveal duration in hours
        #[arg(long, default_value = "24")]
        reveal_hours: u32,
    },
    
    /// Get vote information
    Get {
        /// Vote ID
        vote_id: String,
    },
    
    /// List votes
    List {
        /// Page number
        #[arg(short, long, default_value = "0")]
        page: u32,
        
        /// Page size
        #[arg(long, default_value = "10")]
        size: u32,
        
        /// Filter by status
        #[arg(long)]
        status: Option<String>,
        
        /// Filter by creator
        #[arg(long)]
        creator: Option<String>,
    },
    
    /// Submit a commitment
    Commit {
        /// Vote ID
        vote_id: String,
        
        /// Voter identifier
        #[arg(short = 'v', long)]
        voter: String,
        
        /// Vote value
        #[arg(short = 'V', long)]
        value: String,
        
        /// Salt for commitment
        #[arg(short, long)]
        salt: Option<String>,
    },
    
    /// Submit a reveal
    Reveal {
        /// Vote ID
        vote_id: String,
        
        /// Voter identifier
        #[arg(short = 'v', long)]
        voter: String,
        
        /// Vote value
        #[arg(short = 'V', long)]
        value: String,
        
        /// Salt used in commitment
        #[arg(short, long)]
        salt: String,
    },
    
    /// Get vote results
    Results {
        /// Vote ID
        vote_id: String,
    },
    
    /// Verify vote results
    Verify {
        /// Vote ID
        vote_id: String,
    },
    
    /// List available templates
    Templates,
    
    /// Get template information
    Template {
        /// Template ID
        template_id: String,
    },
    
    /// Health check
    Health,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    init_logging_from_env()?;
    
    let cli = Cli::parse();
    
    if cli.verbose {
        std::env::set_var("RUST_LOG", "debug");
    }
    
    info!("Starting vote CLI");
    
    // Create API client
    let client = client::ApiClient::new(&cli.api_url)?;
    
    // Execute command
    match cli.command {
        Commands::Create {
            title,
            description,
            template,
            params,
            commitment_hours,
            reveal_hours,
        } => {
            create_vote(&client, title, description, template, params, commitment_hours, reveal_hours).await?;
        }
        
        Commands::Get { vote_id } => {
            get_vote(&client, vote_id).await?;
        }
        
        Commands::List { page, size, status, creator } => {
            list_votes(&client, page, size, status, creator).await?;
        }
        
        Commands::Commit { vote_id, voter, value, salt } => {
            commit_vote(&client, vote_id, voter, value, salt).await?;
        }
        
        Commands::Reveal { vote_id, voter, value, salt } => {
            reveal_vote(&client, vote_id, voter, value, salt).await?;
        }
        
        Commands::Results { vote_id } => {
            get_results(&client, vote_id).await?;
        }
        
        Commands::Verify { vote_id } => {
            verify_results(&client, vote_id).await?;
        }
        
        Commands::Templates => {
            list_templates(&client).await?;
        }
        
        Commands::Template { template_id } => {
            get_template(&client, template_id).await?;
        }
        
        Commands::Health => {
            health_check(&client).await?;
        }
    }
    
    Ok(())
}
