use crate::Item;
use crossterm::event;
use crossterm::event::KeyCode;
use crossterm::event::KeyEventKind;
use ratatui::prelude::*;
use ratatui::widgets::*;
use ratatui::Terminal;

#[derive(Debug, Clone, Copy, PartialEq)]
enum Message {
    OpenHelp,
    MoveSelection { is_down: bool },
    JumpSelection { to_start: bool },
    Select,
    ChangeFocus,
    Quit,
}

fn handle_event(event: event::Event) -> Option<Message> {
    Some(match event {
        event::Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
            match key_event.code {
                KeyCode::Char(c) => match c {
                    'j' => Message::MoveSelection { is_down: true },
                    'k' => Message::MoveSelection { is_down: false },
                    'q' => Message::Quit,
                    '?' => Message::OpenHelp,
                    'g' => Message::JumpSelection { to_start: true },
                    'G' => Message::JumpSelection { to_start: false },
                    _ => return None,
                },
                KeyCode::Up => Message::MoveSelection { is_down: false },
                KeyCode::Down => Message::MoveSelection { is_down: true },
                KeyCode::Home => Message::JumpSelection { to_start: true },
                KeyCode::End => Message::JumpSelection { to_start: false },
                KeyCode::Enter => Message::Select,
                KeyCode::Tab => Message::ChangeFocus,
                KeyCode::Esc => Message::Quit,
                _ => return None,
            }
        }
        _ => return None,
    })
}

fn list<'a, I>(title: &'a str, items: I) -> List
where
    I: IntoIterator,
    I::Item: Into<ListItem<'a>>,
{
    ratatui::widgets::List::new(items)
        .block(Block::bordered().title(title))
        .highlight_symbol(">")
        .highlight_style(
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
}

fn draw_information(frame: &mut Frame) -> Rect {
    let layout = Layout::vertical([Constraint::Min(0), Constraint::Length(1)]);
    let [top, bot] = layout.areas(frame.size());
    let info = Paragraph::new("Press ? for help").reversed().centered();
    frame.render_widget(info, bot);
    top
}

type Set<T> = Vec<T>;

pub fn select_multiple<'a>(
    selected: &mut Set<Item<'a>>,
    unselected: &mut Set<Item<'a>>,
    mut terminal: Terminal<impl Backend>,
) -> crate::Result<()> {
    let mut lists = [unselected, selected];
    let mut list_states: [ListState; 2] = Default::default();
    let mut current_selections: [usize; 2] = Default::default();
    let mut current_list = 0;
    loop {
        let current_selection = &mut current_selections[current_list];
        list_states[current_list].select(Some(*current_selection));
        list_states[(current_list + 1) % 2].select(None);

        terminal.draw(|frame| {
            let unselected_list =
                list("Packages to be kept", lists[0].iter().map(|pkg| pkg.name()));

            let selected_list = list(
                "Packages to be removed",
                lists[1].iter().map(|pkg| pkg.name()),
            );

            let list_area = draw_information(frame);

            let layout =
                Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]);
            let [left, right] = layout.areas(list_area);

            let lists = [(left, unselected_list), (right, selected_list)];

            for ((area, list), state) in lists.into_iter().zip(&mut list_states) {
                frame.render_stateful_widget(list, area, state);
            }
        })?;

        let msg = handle_event(event::read()?);
        let Some(msg) = msg else { continue };
        match msg {
            Message::OpenHelp => crate::help_page::help_page(&mut terminal)?,
            Message::Quit => break,
            Message::Select => {
                // Truly, one of the code ever written
                let [ref mut to_add, ref mut to_remove] = lists;
                let mut to_remove = to_remove;
                let mut to_add = to_add;
                if *current_selection == 1 {
                    std::mem::swap(&mut to_remove, &mut to_add);
                }
                if !to_remove.is_empty() {
                    to_add.push(to_remove.remove(*current_selection));
                    to_add.sort()
                }
            }
            Message::JumpSelection { to_start } => {
                *current_selection = if to_start {
                    0
                } else {
                    lists[current_list].len() - 1
                }
            }
            Message::MoveSelection { is_down: true } => {
                *current_selection = current_selection.wrapping_add(1) % lists[current_list].len();
            }
            Message::MoveSelection { is_down: false } => {
                *current_selection = current_selection.wrapping_sub(1)
            }
            Message::ChangeFocus => current_list = (current_list + 1) % 2,
        }
    }
    Ok(())
}
