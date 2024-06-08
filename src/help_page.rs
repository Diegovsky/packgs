use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{backend::Backend, layout::{Constraint, Layout}, text::Line, widgets::*, Terminal};

fn should_quit(event: Event) -> bool {
    if let Event::Key(KeyEvent {
        kind: KeyEventKind::Press,
        code,
        ..
    }) = event
    {
        match code {
            KeyCode::Enter|
            KeyCode::Char('q'|'?') => return true,
            _ => ()
        }
    }
    false
}

pub fn help_page(terminal: &mut Terminal<impl Backend>) -> crate::Result<()> {
    let (keys, meanings): (Vec<_>, Vec<_>) = [
        "g: Go to first item",
        "G: Go to last item",
        "j: Go one item down",
        "k: Go one item up",
        "q: Close without saving",
        "<Return>: Move item to other list",
        "<Tab>: Go to other list",
        "?: Toggle help menu",
    ].map(|line| line.split_once(": ").unwrap()).into_iter().unzip();
    let keys = List::new(keys);
    let meanings = List::new(meanings);
    let layout = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]);
    let block = Block::new().title("Key maps").borders(Borders::ALL);
    loop {
        terminal.draw(|frame| {
            let area = frame.size();
            frame.render_widget(&block, area);

            let area = block.inner(area);
            let [l, r] = layout.areas(area);
            frame.render_widget(&keys, l);
            frame.render_widget(&meanings, r);
        })?;

        if should_quit(event::read()?) {
            break
        }
    }
    Ok(())
}
