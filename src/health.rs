use std::{sync::Arc, time::Duration};

use anyhow::Context;
use tokio::time;

use crate::config::{AppState, Process, State, StepConfig};

pub async fn health_check_daemon(process: Arc<Process>, state: Arc<State>) {
    let mut interval = time::interval(Duration::from_millis(200));
    loop {
        for step in &process.steps {
            match step {
                StepConfig::Serial(s) => match state.get_mut(&s.app) {
                    Some(mut hs) => {
                        if *hs == AppState::STARTED {
                            let health = is_healthy(&s.health_check_url).await;
                            if health {
                                *hs = AppState::RUNNING;
                            };
                        } else if *hs == AppState::RUNNING || *hs == AppState::ERROR {
                            let health = is_healthy(&s.health_check_url).await;
                            if health {
                                *hs = AppState::RUNNING;
                            } else {
                                *hs = AppState::ERROR;
                            };
                        }
                    }
                    None => unreachable!(),
                },
                StepConfig::Parallel(ps) => {
                    for p in ps {
                        match state.get_mut(&p.app) {
                            Some(mut hs) => {
                                if *hs == AppState::STARTED {
                                    let health = is_healthy(&p.health_check_url).await;
                                    if health {
                                        *hs = AppState::RUNNING;
                                    };
                                } else if *hs == AppState::RUNNING || *hs == AppState::ERROR {
                                    let health = is_healthy(&p.health_check_url).await;
                                    if health {
                                        *hs = AppState::RUNNING;
                                    } else {
                                        *hs = AppState::ERROR;
                                    };
                                }
                            }
                            None => unreachable!(),
                        }
                    }
                }
            };
        }
        interval.tick().await;
    }
}

async fn is_healthy(health_check_url: &Option<String>) -> bool {
    match health_check_url {
        Some(health_check_url) => {
            match reqwest::get(health_check_url)
                .await
                .context("Error checking health")
            {
                Ok(r) => r.status().is_success(),
                Err(_) => false,
            }
        }
        None => true,
    }
}
