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

pub enum DeleteMsg {
    Progress {
        message: String,
        done: usize,
        total: usize,
        freed: u64,
    },
    Done {
        count: usize,
        freed: u64,
        trashed: bool,
    },
}

pub type ScanJob = Box<dyn FnOnce(Sender<ScanMsg>) + Send>;
pub type DeleteRunner = Box<dyn FnOnce(Vec<ScanEntry>, Disposition, Sender<DeleteMsg>) + Send>;

pub fn run(mut app: App, scan_job: ScanJob, delete_runner: DeleteRunner) -> io::Result<()> {
    let mut terminal = setup()?;
    let outcome = event_loop(&mut terminal, &mut app, scan_job, delete_runner);
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

fn event_loop(
    terminal: &mut Tui,
    app: &mut App,
    scan_job: ScanJob,
    delete_runner: DeleteRunner,
) -> io::Result<()> {
    let mut scan_job = Some(scan_job);
    let mut delete_runner = Some(delete_runner);
    let mut scan_rx: Option<Receiver<ScanMsg>> = None;
    let mut del_rx: Option<Receiver<DeleteMsg>> = None;

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

        if app.start_scan_requested && scan_rx.is_none() {
            app.start_scan_requested = false;
            app.update_scan("Подготовка…".into(), 0, 0, 0);
            let (tx, rx) = channel();
            scan_rx = Some(rx);
            if let Some(job) = scan_job.take() {
                std::thread::spawn(move || job(tx));
            }
        }
        if let Some(rx) = &scan_rx {
            drain_scan(rx, app);
        }

        if app.pending_delete.is_some() && del_rx.is_none() {
            if let Some(disposition) = app.pending_delete.take() {
                let entries = app.selected_entries();
                app.update_delete("Подготовка…".into(), 0, entries.len(), 0);
                let (tx, rx) = channel();
                del_rx = Some(rx);
                if let Some(runner) = delete_runner.take() {
                    std::thread::spawn(move || runner(entries, disposition, tx));
                }
            }
        }
        if let Some(rx) = &del_rx {
            drain_delete(rx, app);
        }

        if app.should_quit {
            return Ok(());
        }
    }
}

fn drain_scan(rx: &Receiver<ScanMsg>, app: &mut App) {
    while let Ok(msg) = rx.try_recv() {
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

fn drain_delete(rx: &Receiver<DeleteMsg>, app: &mut App) {
    while let Ok(msg) = rx.try_recv() {
        match msg {
            DeleteMsg::Progress {
                message,
                done,
                total,
                freed,
            } => app.update_delete(message, done, total, freed),
            DeleteMsg::Done {
                count,
                freed,
                trashed,
            } => app.set_summary(count, freed, trashed),
        }
    }
}
