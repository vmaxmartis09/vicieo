use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "vicieo")]
#[command(about = "Vicieo: Lightweight High-Performance Multiplatform Video AI Orchestrator", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Start the high-performance Rust Backend API server (Default: port 9000)
    Serve {
        #[arg(short, long, default_value_t = 9000)]
        port: u16,
    },
    /// Inspect project and workspace information
    Info {
        #[arg(short, long, default_value = "soc-nho-tv")]
        project: String,
    },
    /// Manage multi-project workspaces
    Project {
        #[command(subcommand)]
        action: ProjectAction,
    },
    /// Render video or HyperFrames composition via FFmpeg & Browser
    Render {
        #[arg(short, long, default_value = "soc-nho-tv")]
        project: String,
        #[arg(short, long)]
        script: String,
        #[arg(short, long)]
        output: Option<String>,
    },
    /// Query generative AI models (Gemini, Claude, OpenAI)
    Ai {
        #[arg(short, long)]
        prompt: String,
        #[arg(long, default_value = "gemini")]
        provider: String,
        #[arg(short, long)]
        model: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
pub enum ProjectAction {
    List,
    Switch { id: String },
}

pub async fn handle_project_cmd(action: ProjectAction) -> anyhow::Result<()> {
    match action {
        ProjectAction::List => {
            println!("Available projects in workspace:");
            let projects = crate::workspace::list_projects();
            for p in projects {
                println!(" - {} ({:?})", p.id, p.name);
            }
        }
        ProjectAction::Switch { id } => {
            println!("Switched active project to: {}", id);
        }
    }
    Ok(())
}
