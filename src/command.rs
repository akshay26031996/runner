use std::{fs::File, process::Command, sync::Arc, time::Duration};

use tokio::time::sleep;

<<<<<<< HEAD
use crate::{config::{AppState, Process, State, StepConfig}, health};

fn run_command(command: &String) {
    Command::new("sh")
        .arg("-c")
        .arg(command)
        .stdout(File::create("/tmp/runner.log").expect("failed to open log"))
        .stderr(File::create("/tmp/runner-err.log").expect("failed to open log"))
=======
use crate::config::{AppState, Process, State, StepConfig};

fn run_command(app: &String, command: &String) {
    let log_name = format!("/tmp/{}.log", app);
    let log = File::create(log_name).expect("failed to open log");
    Command::new("sh")
        .arg("-c")
        .arg(command)
        .stdout(log)
>>>>>>> ca159e0622c5550a49851e7f40a21a46406afaaa
        .spawn()
        .expect("Failed to run command");
}

<<<<<<< HEAD
fn run_command_sync(command: &String) {
    Command::new("sh")
        .arg("-c")
        .arg(command)
        .stdout(File::create("/tmp/runner.log").expect("failed to open log"))
        .stderr(File::create("/tmp/runner-err.log").expect("failed to open log"))
        .output()
        .expect("Failed to run command");
}

=======
>>>>>>> ca159e0622c5550a49851e7f40a21a46406afaaa
pub async fn start_processes(process: Arc<Process>, state: Arc<State>) {
    for step in &process.steps {
        let mut started_apps = vec![];
        match step {
<<<<<<< HEAD
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
=======
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
>>>>>>> ca159e0622c5550a49851e7f40a21a46406afaaa
                }
            }
        };

        let mut is_started = false;
        while !is_started {
            is_started = true;
            for app in &started_apps {
<<<<<<< HEAD
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
=======
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
>>>>>>> ca159e0622c5550a49851e7f40a21a46406afaaa
