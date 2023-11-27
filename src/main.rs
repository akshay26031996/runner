mod command;
mod config;
mod health;
mod ui;

use std::{sync::Arc, time::Duration};

use clap::Parser;

use config::{AppState, Process, State, StepConfig};
use tokio::time::sleep;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    config: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    console_subscriber::init();
    // Configuration - START
    let cli = Cli::parse();

    let file = std::fs::File::open(cli.config)?;
    let reader = std::io::BufReader::new(file);
    let process: Process = serde_json::from_reader(reader)?;
    let process = Arc::new(process);
    // Configuration - END

    // Initialization - START
    let state = Arc::new(State::new());

    for step in &process.steps {
        match step {
            StepConfig::Serial(s) => {
                state.insert(s.app.clone(), AppState::PENDING);
            }
            StepConfig::Parallel(ps) => {
                for p in ps {
                    state.insert(p.app.clone(), AppState::PENDING);
                }
            }
        }
    }

    health::health_check(Arc::clone(&process), Arc::clone(&state)).await;
    sleep(Duration::from_secs(1)).await;
    // Initialization - END

    // Command Section - START
    let command_state = Arc::clone(&state);
    let command_process = Arc::clone(&process);
    tokio::spawn(async move {
        command::start_processes(command_process, command_state).await;
    });
    // Command Section - END

    // Health Section - START
    let health_state = Arc::clone(&state);
    let health_process = Arc::clone(&process);
    tokio::spawn(async move {
        health::health_check_daemon(health_process, health_state).await;
    });
    // Health Section - END

    // UI Section - START
    ui::render(Arc::clone(&process), Arc::clone(&state)).await?;
    // UI Section - END
    Ok(())
}
