use clap::{Parser, Subcommand, Args};
use serde_json::json;
use crate::service::{VoteService, VoteServiceImpl};
use crate::core::template::TemplateRegistry;
use crate::store::{VoteStore, memory::MemoryVoteStore};
use std::sync::Arc;
use crate::model::vote::*;

#[derive(Parser, Debug)]
#[command(name = "ddv")] 
#[command(about = "Decentralized decision vote CLI", long_about = None)]
pub struct Cli {
    /// Run HTTP API server instead of CLI actions
    #[arg(long, default_value_t=false)]
    pub api_mode: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Create(CreateArgs),
    Commit(CommitArgs),
    Reveal(RevealArgs),
}

#[derive(Args, Debug)]
pub struct CreateArgs {
    #[arg(long)] pub title: String,
    #[arg(long)] pub options: Vec<String>,
    #[arg(long, default_value_t=0)] pub commit_start: u64,
    #[arg(long)] pub commit_end: u64,
    #[arg(long)] pub reveal_start: u64,
    #[arg(long)] pub reveal_end: u64,
    #[arg(long)] pub participants: Vec<String>,
    #[arg(long, default_value="option_index")] pub value_template: String,
    #[arg(long, default_value_t=0)] pub template_max: u64,
}

#[derive(Args, Debug)]
pub struct CommitArgs {
    #[arg(long)] pub vote_id: String,
    #[arg(long)] pub voter: String,
    #[arg(long)] pub vote_value: u64,
    #[arg(long)] pub salt_hex: String,
}

#[derive(Args, Debug)]
pub struct RevealArgs {
    #[arg(long)] pub vote_id: String,
    #[arg(long)] pub voter: String,
    #[arg(long)] pub vote_value: u64,
    #[arg(long)] pub salt_hex: String,
}

pub fn parse_args() -> Cli { Cli::parse() }

pub async fn execute_cli(cli: Cli) -> i32 {
    // build in-memory service and registry to reuse core logic
    let store: Arc<dyn VoteStore> = Arc::new(MemoryVoteStore::default());
    let mut reg = TemplateRegistry::new();
    reg.register(crate::core::template::BitTemplate);
    reg.register(crate::core::template::OptionIndexTemplate);
    reg.register(crate::core::template::StringTemplate);
    let service = VoteServiceImpl::new(store.clone(), Arc::new(reg));
    match cli.command {
        Some(Commands::Create(args)) => {
            let cfg = VoteConfig {
                title: args.title,
                description: None,
                options: args.options.clone(),
                commit_start_height: args.commit_start,
                commit_end_height: args.commit_end,
                reveal_start_height: args.reveal_start,
                reveal_end_height: args.reveal_end,
                participants: args.participants.clone(),
                value_template: args.value_template,
                template_params: json!({"max": args.template_max}),
            };
            match service.create_vote(cfg).await {
                Ok(id) => { println!("{}", id); 0 }
                Err(e) => { eprintln!("error: {}", e); 1 }
            }
        }
        Some(Commands::Commit(args)) => {
            match service.commit(&args.vote_id, &args.voter, json!(args.vote_value), args.salt_hex).await {
                Ok(r) => { println!("{}", r.commitment_hex); 0 }
                Err(e) => { eprintln!("error: {}", e); 2 }
            }
        }
        Some(Commands::Reveal(args)) => {
            match service.reveal(&args.vote_id, &args.voter, json!(args.vote_value), args.salt_hex).await {
                Ok(r) => { println!("{}", r.accepted); 0 }
                Err(e) => { eprintln!("error: {}", e); 3 }
            }
        }
        None => { eprintln!("no command provided"); 64 }
    }
}
