use crate::app::App;
use crate::event::map_key;
use crate::render::render;
use crossterm::event::{self, Event, KeyEventKind};
use crossterm::{execute, terminal};
use oswam_core::scan::ScanEntry;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io::{self, Stdout};

type Tui = Terminal<CrosstermBackend<Stdout>>;

pub fn run(app: &mut App) -> io::Result<Option<Vec<ScanEntry>>> {
    let mut terminal = setup()?;
    let outcome = event_loop(&mut terminal, app);
    restore(&mut terminal)?;
    outcome
}

pub fn selected_entries(app: &App) -> Vec<ScanEntry> {
    let mut out = Vec::new();
    for (ci, cat) in app.result.categories.iter().enumerate() {
        for (ei, entry) in cat.entries.iter().enumerate() {
            if app.is_selected(ci, ei) {
                out.push(entry.clone());
            }
        }
    }
    out
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

fn event_loop(terminal: &mut Tui, app: &mut App) -> io::Result<Option<Vec<ScanEntry>>> {
    loop {
        terminal.draw(|f| render(f, app))?;
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                if let Some(mapped) = map_key(key) {
                    app.on_key(mapped);
                }
            }
        }
        if app.should_quit {
            return Ok(None);
        }
        if app.proceed_requested {
            return Ok(Some(selected_entries(app)));
        }
    }
}
