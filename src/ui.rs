use std::sync::Arc;

use crossterm::{
    cursor,
    event::{Event, EventStream, KeyCode, KeyEvent, KeyModifiers},
    execute, queue,
    style::{self, Stylize},
    terminal,
};

use futures::{FutureExt, StreamExt};
use tokio::{
    sync::mpsc::{Receiver, Sender},
    time::{self, Duration},
};

use crate::{
    command,
    config::{AppState, Process, State, StepConfig},
};

#[derive(Debug, PartialEq)]
pub enum UiEvent {
    Tick,
    Cancelled,
    Input(String, bool),
}

async fn genereate_ui_events(tx: Sender<UiEvent>) {
    let mut interval = time::interval(Duration::from_millis(1009));
    let mut reader = EventStream::new();
    let ctrl_c_event = Event::Key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL));
    let quit_event = Event::Key(KeyCode::Char('q').into());
    let input_event = Event::Key(KeyCode::Char('/').into());
    let enter_event = Event::Key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    let mut input = String::new();
    let mut is_input_mode = false;
    loop {
        let delay = interval.tick().fuse();
        let event = reader.next().fuse();
        tokio::select! {
            _ = delay => {
                tx.send(UiEvent::Tick).await.expect("Can send events");
            },
            maybe_event = event => {
                match maybe_event {
                    Some(Ok(e)) => {
                        if is_input_mode {
                            if let Event::Key(KeyEvent { code, .. }) = e {
                                match code {
                                    KeyCode::Esc => {
                                        input.clear();
                                        tx.send(UiEvent::Input(input.clone(), true)).await.expect("Can send events");
                                        is_input_mode = false;
                                    }
                                    KeyCode::Backspace => {
                                        input.pop();
                                        tx.send(UiEvent::Input(input.clone(), false)).await.expect("Can send events");
                                        if input.is_empty() {
                                            is_input_mode = false;
                                        }
                                    }
                                    KeyCode::Char(c) => {
                                        input.push(c);
                                        tx.send(UiEvent::Input(input.clone(), false)).await.expect("Can send events");
                                    }
                                    _ => {}
                                }
                            }
                        }

                        if e == input_event {
                            is_input_mode = true;
                        } else if e == enter_event && is_input_mode {
                            tx.send(UiEvent::Input(input.clone(), true)).await.expect("Can send events");
                            input.clear();
                            is_input_mode = false;
                        }
                        else if e == quit_event || e == ctrl_c_event {
                            tx.send(UiEvent::Cancelled).await.expect("Can send events");
                            break;
                        }
                    },
                    Some(Err(_)) => {
                        tx.send(UiEvent::Cancelled).await.expect("Can send events");
                        break;
                    },
                    None => {}
                }
            }
        };
    }
}

async fn render_state<W>(
    process: Arc<Process>,
    state: Arc<State>,
    rx: &mut Receiver<UiEvent>,
    w: &mut W,
) -> anyhow::Result<()>
where
    W: std::io::Write,
{
    let mut input = String::new();

    let mut max_length = 0;
    for pr in &process.steps {
        match pr {
            StepConfig::Serial(config) => {
                if max_length < config.app.len() {
                    max_length = config.app.len();
                }
            }
            StepConfig::Parallel(p) => {
                for config in p {
                    if max_length < config.app.len() {
                        max_length = config.app.len();
                    }
                }
            }
        }
    }
    max_length += 5;

    loop {
        let event = rx.recv().await.expect("Can read events");

        if let UiEvent::Input(user_input, is_input_complete) = event {
            input = user_input;
            if is_input_complete {
                if !input.is_empty() {
                    let restart_process = Arc::clone(&process);
                    let restart_state = Arc::clone(&state);
                    let restart_input = input.clone();
                    tokio::spawn(async move {
                        command::re_start_processes(restart_process, restart_state, restart_input)
                            .await;
                    });
                }
                input.clear();
            }
        } else if event == UiEvent::Cancelled {
            break;
        }

        queue!(
            w,
            terminal::Clear(terminal::ClearType::All),
            cursor::RestorePosition,
            cursor::MoveToNextLine(1)
        )?;
        for step in &process.steps {
            match step {
                StepConfig::Serial(s) => {
                    render_app(&s.app, state.get(&s.app).unwrap().value(), w, max_length)?;
                }
                StepConfig::Parallel(ps) => {
                    for p in ps {
                        render_app(&p.app, state.get(&p.app).unwrap().value(), w, max_length)?;
                    }
                }
            };
        }
        if !input.is_empty() {
            queue!(
                w,
                style::PrintStyledContent("🔍 ".bold()),
                style::Print(&input),
                cursor::MoveToNextLine(1)
            )?;
        }
        w.flush()?;
    }
    Ok(())
}

fn render_app<W>(app: &String, app_state: &AppState, w: &mut W, max_length: usize) -> anyhow::Result<()>
where
    W: std::io::Write,
{
    match app_state {
        AppState::PENDING => queue!(
            w,
            style::Print(format!("{:<max_length$}", app)),
            style::Print("█ "),
            style::PrintStyledContent("🕓".bold()),
            style::Print(" █"),
            cursor::MoveToNextLine(1)
        )?,
        AppState::STARTED => queue!(
            w,
            style::Print(format!("{:<max_length$}", app)),
            style::Print("█ "),
            style::PrintStyledContent("🟡".bold()),
            style::Print(" █"),
            cursor::MoveToNextLine(1),
        )?,
        AppState::RUNNING => queue!(
            w,
            style::Print(format!("{:<max_length$}", app)),
            style::Print("█ "),
            style::PrintStyledContent("✅".bold()),
            style::Print(" █"),
            cursor::MoveToNextLine(1),
        )?,
        AppState::ERROR => queue!(
            w,
            style::Print(format!("{:<max_length$}", app)),
            style::Print("█ "),
            style::PrintStyledContent("❌".bold()),
            style::Print(" █"),
            cursor::MoveToNextLine(1),
        )?,
        AppState::RESTART => queue!(
            w,
            style::Print(format!("{:<max_length$}", app)),
            style::Print("█ "),
            style::PrintStyledContent("⭕️".bold()),
            style::Print(" █"),
            cursor::MoveToNextLine(1),
        )?,
    };
    Ok(())
}

pub async fn render(process: Arc<Process>, state: Arc<State>) -> anyhow::Result<()> {
    let mut stdout = std::io::stdout();
    let (tx, mut rx) = tokio::sync::mpsc::channel(1);

    execute!(stdout, terminal::EnterAlternateScreen, cursor::Hide, cursor::SavePosition,)?;
    terminal::enable_raw_mode().expect("Can run in raw mode");

    tokio::spawn(async move {
        genereate_ui_events(tx).await;
    });

    render_state(process, state, &mut rx, &mut stdout).await?;

    execute!(
        stdout,
        style::ResetColor,
        cursor::Show,
        terminal::LeaveAlternateScreen
    )?;
    terminal::disable_raw_mode().expect("Can run in raw mode");
    Ok(())
}
