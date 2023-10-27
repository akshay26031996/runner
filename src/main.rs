mod command;
mod config;
mod health;
mod ui;

use std::sync::Arc;

use clap::Parser;

use config::{AppState, Process, State, StepConfig};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    config: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
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
    let mut stdout = std::io::stdout();
    let (tx, mut rx) = tokio::sync::mpsc::channel(1);
    tokio::spawn(async move {
        ui::genereate_ui_events(tx).await;
    });
    ui::render_state(
        Arc::clone(&process),
        Arc::clone(&state),
        &mut rx,
        &mut stdout,
    )
    .await?;
    // UI Section - END
    Ok(())
}
