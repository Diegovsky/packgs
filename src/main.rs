use alpm::Package;
use crossterm::terminal::disable_raw_mode;
use crossterm::terminal::EnterAlternateScreen;
use crossterm::terminal::LeaveAlternateScreen;
use crossterm::ExecutableCommand;
use ratatui::prelude::*;
use ratatui::Terminal;
use std::collections::HashSet;
use std::ffi::OsStr;
use std::fs::File;
use std::io::stdout;
use std::io::BufRead;
use std::io::BufReader;

use color_eyre::{config::HookBuilder, Result};

mod packages;
mod select_item;
mod help_page;


fn main() -> Result<()> {
    // setup terminal
    init_error_hooks()?;
    let terminal = init_terminal()?;

    // create app and run it
    app(terminal, std::env::args_os().nth(1).as_deref())?;

    restore_terminal()?;

    Ok(())
}

fn init_error_hooks() -> color_eyre::Result<()> {
    let (panic, error) = HookBuilder::default().into_hooks();
    let panic = panic.into_panic_hook();
    let error = error.into_eyre_hook();
    color_eyre::eyre::set_hook(Box::new(move |e| {
        let _ = restore_terminal();
        error(e)
    }))?;
    std::panic::set_hook(Box::new(move |info| {
        let _ = restore_terminal();
        panic(info);
    }));
    Ok(())
}

fn init_terminal() -> color_eyre::Result<Terminal<impl Backend>> {
    crossterm::terminal::enable_raw_mode()?;
    std::io::stdout().execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout());
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

fn restore_terminal() -> color_eyre::Result<()> {
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}

#[derive(Clone, Copy, Debug)]
struct Item<'a>(&'a Package);

impl<'a> Eq for Item<'a> {}

impl<'a> Ord for Item<'a> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name().cmp(other.name())
    }
}

impl<'a> Item<'a> {
    fn name(&self) -> &str {
        self.0.name()
    }
}

impl<'a> PartialEq for Item<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.name() == other.name()
    }
}

impl std::cmp::PartialOrd for Item<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

fn app(terminal: Terminal<impl Backend>, filename: Option<&OsStr>) -> Result<()> {
    let mut arch_pkgs = packages::ArchPackages::new()?;
    let selected = BufReader::new(File::open(filename.unwrap_or(OsStr::new("selected.txt")))?);
    let selected: HashSet<String> = selected.lines().map(|l| l.unwrap()).collect();
    let (selected, unselected) = arch_pkgs
        .get_root_packages()
        .partition(|item| selected.contains(item.name()));
    let selections = select_item::select_multiple(selected, unselected, terminal);
    /*
    let mut f = BufWriter::new(File::create("selected.txt")?);
    for l in selections {
        f.write_all(l.as_bytes())?;
        f.write_all(b"\n")?;
    }
     */

    Ok(())
}
