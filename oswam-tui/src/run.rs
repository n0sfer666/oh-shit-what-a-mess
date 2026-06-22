use crate::app::App;
use crate::event::map_key;
use crate::render::render;
use crossterm::event::{self, Event, KeyEventKind};
use crossterm::{execute, terminal};
use oswam_core::delete::Disposition;
use oswam_core::scan::{ScanEntry, ScanResult};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io::{self, Stdout};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::Duration;

type Tui = Terminal<CrosstermBackend<Stdout>>;

pub enum ScanMsg {
    Progress {
        message: String,
        done: usize,
        total: usize,
        bytes: u64,
    },
    Done(ScanResult),
}

pub type ScanJob = Box<dyn FnOnce(Sender<ScanMsg>) + Send>;

pub struct Decision {
    pub entries: Vec<ScanEntry>,
    pub disposition: Disposition,
}

pub fn run(mut app: App, job: ScanJob) -> io::Result<Option<Decision>> {
    let mut terminal = setup()?;
    let outcome = event_loop(&mut terminal, &mut app, job);
    restore(&mut terminal)?;
    outcome
}

fn setup() -> io::Result<Tui> {
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, terminal::EnterAlternateScreen)?;
    Terminal::new(CrosstermBackend::new(stdout))
}

fn restore(terminal: &mut Tui) -> io::Result<()> {
    terminal::disable_raw_mode()?;
    execute!(terminal.backend_mut(), terminal::LeaveAlternateScreen)?;
    terminal.show_cursor()
}

fn event_loop(terminal: &mut Tui, app: &mut App, job: ScanJob) -> io::Result<Option<Decision>> {
    let mut job = Some(job);
    let mut rx: Option<Receiver<ScanMsg>> = None;
    loop {
        terminal.draw(|f| render(f, app))?;

        if event::poll(Duration::from_millis(120))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    if let Some(mapped) = map_key(key) {
                        app.on_key(mapped);
                    }
                }
            }
        }

        if app.start_scan_requested && rx.is_none() {
            app.start_scan_requested = false;
            app.update_scan("Подготовка…".into(), 0, 0, 0);
            let (tx, receiver) = channel();
            rx = Some(receiver);
            if let Some(job) = job.take() {
                std::thread::spawn(move || job(tx));
            }
        }

        if let Some(receiver) = &rx {
            drain(receiver, app);
        }

        if app.should_quit {
            return Ok(None);
        }
        if let Some(disposition) = app.decision {
            return Ok(Some(Decision {
                entries: app.selected_entries(),
                disposition,
            }));
        }
    }
}

fn drain(receiver: &Receiver<ScanMsg>, app: &mut App) {
    while let Ok(msg) = receiver.try_recv() {
        match msg {
            ScanMsg::Progress {
                message,
                done,
                total,
                bytes,
            } => app.update_scan(message, done, total, bytes),
            ScanMsg::Done(result) => app.set_result(result),
        }
    }
}
