pub mod ai;
pub mod audio_mastering;
pub mod caption;
pub mod cli;
pub mod intelligence;
pub mod schema;
pub mod server;
pub mod social;
pub mod video;
pub mod workspace;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Commands};
use colored::*;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    dotenvy::dotenv().ok();

    let cli = Cli::parse();

    match cli.command {
        Commands::Serve { port } => {
            println!("{}", format!("Starting ViciEO Rust Backend on Port {}", port).green().bold());
            server::run_server(port).await?;
        }
        Commands::Info { project } => {
            println!("{}", "=== VICIEO CORE ENGINE (RUST NATIVE) ===".green().bold());
            let root = workspace::repo_root();
            let act_proj = workspace::active_project_id();
            println!("Repo Root:      {}", root.display());
            println!("Active Project: {}", act_proj.cyan().bold());
            let projects = workspace::list_projects();
            println!("Available Projects ({}):", projects.len());
            for p in projects {
                println!(" - {} ({:?})", p.id.yellow(), p.name.unwrap_or_default());
            }
            println!("Status:         {}", "Ready (Lightweight Native Runtime)".green());
        }
        Commands::Project { action } => {
            cli::handle_project_cmd(action).await?;
        }
        Commands::Render { project, script, output } => {
            video::render_pipeline(&project, &script, output.as_deref()).await?;
        }
        Commands::Ai { prompt, provider, model } => {
            ai::generate_text(&prompt, &provider, model.as_deref()).await?;
        }
    }

    Ok(())
}
