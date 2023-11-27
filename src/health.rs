use std::{sync::Arc, time::Duration};

use anyhow::Context;
use dashmap::try_result::TryResult;
use tokio::time;

use crate::config::{AppState, Process, State, StepConfig};

pub async fn health_check_daemon(process: Arc<Process>, state: Arc<State>) {
    let mut interval = time::interval(Duration::from_millis(2000));
    loop {
        health_check(Arc::clone(&process), Arc::clone(&state)).await;
        interval.tick().await;
    }
}

pub async fn health_check(process: Arc<Process>, state: Arc<State>) {
    for step in &process.steps {
        match step {
            StepConfig::Serial(s) => match state.try_get_mut(&s.app) {
                TryResult::Present(mut hs) => {
                    if *hs == AppState::STARTED || *hs == AppState::PENDING {
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
                TryResult::Absent => {}
                TryResult::Locked => {}
            },
            StepConfig::Parallel(ps) => {
                for p in ps {
                    match state.try_get_mut(&p.app) {
                        TryResult::Present(mut hs) => {
                            if *hs == AppState::STARTED || *hs == AppState::PENDING {
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
                        TryResult::Absent => {}
                        TryResult::Locked => {}
                    }
                }
            }
        };
    }
}

pub async fn is_healthy(health_check_url: &Option<String>) -> bool {
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
