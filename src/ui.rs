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

use crate::config::{AppState, Process, State, StepConfig};

pub enum UiEvent {
    Tick,
    Cancelled,
}

pub async fn genereate_ui_events(tx: Sender<UiEvent>) {
    let mut interval = time::interval(Duration::from_millis(200));
    let mut reader = EventStream::new();
    let ctrl_c = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
    loop {
        let delay = interval.tick().fuse();
        let event = reader.next().fuse();
        tokio::select! {
            _ = delay => {
                tx.send(UiEvent::Tick).await.expect("Can send events");
            },
            maybe_event = event => {
                match maybe_event {
                    Some(Ok(event)) => {
                        if event == Event::Key(KeyCode::Char('q').into()) || event == Event::Key(ctrl_c) {
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

pub async fn render_state<W>(
    process: Arc<Process>,
    state: Arc<State>,
    rx: &mut Receiver<UiEvent>,
    w: &mut W,
) -> anyhow::Result<()>
where
    W: std::io::Write,
{
    execute!(w, terminal::EnterAlternateScreen, cursor::SavePosition,)?;
    terminal::enable_raw_mode().expect("Can run in raw mode");
    loop {
        match rx.recv().await.expect("Can read events") {
            UiEvent::Tick => {
                queue!(
                    w,
                    terminal::Clear(terminal::ClearType::All),
                    cursor::RestorePosition,
                    cursor::MoveToNextLine(1)
                )?;
                for step in &process.steps {
                    match step {
                        StepConfig::Serial(s) => {
                            render_app(&s.app, state.get(&s.app).unwrap().value(), w)?;
                        }
                        StepConfig::Parallel(ps) => {
                            for p in ps {
                                render_app(&p.app, state.get(&p.app).unwrap().value(), w)?;
                            }
                        }
                    };
                }
                w.flush()?;
            }
            UiEvent::Cancelled => {
                break;
            }
        };
    }
    execute!(
        w,
        style::ResetColor,
        cursor::Show,
        terminal::LeaveAlternateScreen
    )?;
    terminal::disable_raw_mode().expect("Can run in raw mode");
    Ok(())
}

fn render_app<W>(app: &String, app_state: &AppState, w: &mut W) -> anyhow::Result<()>
where
    W: std::io::Write,
{
    match app_state {
        AppState::PENDING => queue!(
            w,
            style::Print(app),
            style::Print("\t\t█ "),
            style::PrintStyledContent("🕓".bold()),
            style::Print(" █"),
            cursor::MoveToNextLine(1)
        )?,
        AppState::STARTED => queue!(
            w,
            style::Print(app),
            style::Print("\t\t█ "),
            style::PrintStyledContent("🟡".bold()),
            style::Print(" █"),
            cursor::MoveToNextLine(1),
        )?,
        AppState::RUNNING => queue!(
            w,
            style::Print(app),
            style::Print("\t\t█ "),
            style::PrintStyledContent("✅".bold()),
            style::Print(" █"),
            cursor::MoveToNextLine(1),
        )?,
        AppState::ERROR => queue!(
            w,
            style::Print(app),
            style::Print("\t\t█ "),
            style::PrintStyledContent("❌".bold()),
            style::Print(" █"),
            cursor::MoveToNextLine(1),
        )?,
    };
    Ok(())
}
