use std::{fs::File, process::Command, sync::Arc, time::Duration};

use tokio::time::sleep;

use crate::{config::{AppState, Process, State, StepConfig}, health};

fn run_command(command: &String) {
    Command::new("sh")
        .arg("-c")
        .arg(command)
        .stdout(File::create("/tmp/runner.log").expect("failed to open log"))
        .stderr(File::create("/tmp/runner-err.log").expect("failed to open log"))
        .spawn()
        .expect("Failed to run command");
}

fn run_command_sync(command: &String) {
    Command::new("sh")
        .arg("-c")
        .arg(command)
        .stdout(File::create("/tmp/runner.log").expect("failed to open log"))
        .stderr(File::create("/tmp/runner-err.log").expect("failed to open log"))
        .output()
        .expect("Failed to run command");
}

pub async fn start_processes(process: Arc<Process>, state: Arc<State>) {
    for step in &process.steps {
        let mut started_apps = vec![];
        match step {
            StepConfig::Serial(s) => match state.get_mut(&s.app) {
                Some(mut hs) => {
                    if *hs != AppState::RUNNING {
                        run_command(&s.start_command);
                        *hs = AppState::STARTED;
                        started_apps.push(s);
                    }
                }
                None => unreachable!(),
            }
            StepConfig::Parallel(ps) => {
                for p in ps {
                    match state.get_mut(&p.app) {
                        Some(mut hs) => {
                            if *hs != AppState::RUNNING {
                                run_command(&p.start_command);
                                *hs = AppState::STARTED;
                                started_apps.push(p);
                            }
                        }
                        None => unreachable!(),
                    }
                }
            }
        };

        let mut is_started = false;
        while !is_started {
            is_started = true;
            for app in &started_apps {
                if !health::is_healthy(&app.health_check_url).await {
                    is_started = false;
                }
            }
            sleep(Duration::from_millis(500)).await;
        }
    }
}

pub async fn re_start_processes(process: Arc<Process>, state: Arc<State>, app_id: String) {
    for step in &process.steps {
        match step {
            StepConfig::Serial(s) => {
                if s.app == app_id {
                    *state.get_mut(&s.app).unwrap() = AppState::RESTART;
                    run_command_sync(&s.start_command);
                    *state.get_mut(&s.app).unwrap() = AppState::STARTED;
                }
            }
            StepConfig::Parallel(ps) => {
                for p in ps {
                    if p.app == app_id {
                        *state.get_mut(&p.app).unwrap() = AppState::RESTART;
                        run_command_sync(&p.start_command);
                        *state.get_mut(&p.app).unwrap() = AppState::STARTED;
                    }
                }
            }
        };
    }
}
