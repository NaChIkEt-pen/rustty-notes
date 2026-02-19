use edtui::{EditorEventHandler, EditorState, EditorTheme, EditorView, LineNumbers};
use ratatui::{
    layout::{Constraint, Layout},
    style::{Color, Style, Stylize},
    widgets::{Block, BorderType, Widget},
    DefaultTerminal, Frame,
};

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    ratatui::run(app)?;
    Ok(())
}

fn app(terminal: &mut DefaultTerminal) -> std::io::Result<()> {
    let mut state = EditorState::default();
    let mut event_handler = EditorEventHandler::default();

    loop {
        terminal.draw(|frame| render(frame, &mut state))?;

        let event = crossterm::event::read()?;
        if event.is_key_press() {
            if let crossterm::event::Event::Key(key_event) = event {
                if key_event.code == crossterm::event::KeyCode::Esc
                    || (key_event.code == crossterm::event::KeyCode::Char('q')
                        && key_event.modifiers == crossterm::event::KeyModifiers::CONTROL)
                {
                    break Ok(());
                }
                event_handler.on_key_event(key_event, &mut state);
            }
        }
    }
}

fn render(frame: &mut Frame, state: &mut EditorState) {
    let [border_area] = Layout::vertical([Constraint::Fill(1)])
        .margin(1)
        .areas(frame.area());

    let [inner_area_left] = Layout::vertical([Constraint::Fill(1)])
        .margin(1)
        .areas(border_area);

    Block::bordered()
        .border_type(BorderType::Rounded)
        .fg(Color::Red)
        .render(border_area, frame.buffer_mut());

    let theme = EditorTheme::default()
        .base(Style::default().bg(Color::Reset).fg(Color::Reset))
        .cursor_style(Style::default().bg(Color::White).fg(Color::Black))
        .line_numbers_style(Style::default().fg(Color::Gray));

    EditorView::new(state)
        .theme(theme)
        .line_numbers(LineNumbers::Absolute)
        .wrap(true)
        .syntax_highlighter(None)
        .tab_width(2)
        .render(inner_area_left, frame.buffer_mut());
}
