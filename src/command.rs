use std::{fs::File, process::Command, sync::Arc, time::Duration};

use tokio::time::sleep;

use crate::config::{AppState, Process, State, StepConfig};

fn run_command(app: &String, command: &String) {
    let log_name = format!("/tmp/{}.log", app);
    let log = File::create(log_name).expect("failed to open log");
    Command::new("sh")
        .arg("-c")
        .arg(command)
        .stdout(log)
        .spawn()
        .expect("Failed to run command");
}

pub async fn start_processes(process: Arc<Process>, state: Arc<State>) {
    for step in &process.steps {
        let mut started_apps = vec![];
        match step {
            StepConfig::Serial(s) => {
                run_command(&s.app, &s.start_command);
                *state.get_mut(&s.app).unwrap() = AppState::STARTED;
                started_apps.push(&s.app);
            }
            StepConfig::Parallel(ps) => {
                for p in ps {
                    run_command(&p.app, &p.start_command);
                    *state.get_mut(&p.app).unwrap() = AppState::STARTED;
                    started_apps.push(&p.app);
                }
            }
        };

        let mut is_started = false;
        while !is_started {
            is_started = true;
            for app in &started_apps {
                let cs = state.try_get(*app);
                if cs.is_locked() || *cs.unwrap() != AppState::RUNNING {
                    is_started = false;
                    break;
                }
            }
            sleep(Duration::from_millis(100)).await;
        }
    }
}
