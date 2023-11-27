mod command;
mod config;
mod health;
mod ui;

<<<<<<< HEAD
use std::{sync::Arc, time::Duration};
=======
use std::sync::Arc;
>>>>>>> ca159e0622c5550a49851e7f40a21a46406afaaa

use clap::Parser;

use config::{AppState, Process, State, StepConfig};
<<<<<<< HEAD
use tokio::time::sleep;
=======
>>>>>>> ca159e0622c5550a49851e7f40a21a46406afaaa

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    config: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
<<<<<<< HEAD
    console_subscriber::init();
=======
>>>>>>> ca159e0622c5550a49851e7f40a21a46406afaaa
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
<<<<<<< HEAD

    health::health_check(Arc::clone(&process), Arc::clone(&state)).await;
    sleep(Duration::from_secs(1)).await;
=======
>>>>>>> ca159e0622c5550a49851e7f40a21a46406afaaa
    // Initialization - END

    // Command Section - START
    let command_state = Arc::clone(&state);
    let command_process = Arc::clone(&process);
<<<<<<< HEAD

=======
>>>>>>> ca159e0622c5550a49851e7f40a21a46406afaaa
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
<<<<<<< HEAD
    ui::render(Arc::clone(&process), Arc::clone(&state)).await?;
=======
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
>>>>>>> ca159e0622c5550a49851e7f40a21a46406afaaa
    // UI Section - END
    Ok(())
}
