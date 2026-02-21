#![allow(clippy::collapsible_if)]
use std::{collections::HashMap, fs};

use edtui::{
    EditorEventHandler, EditorState, EditorTheme, EditorView, LineNumbers, SyntaxHighlighter,
};
use ratatui::{
    layout::{Constraint, Layout},
    style::{Color, Style},
    widgets::{Block, BorderType, Widget},
    DefaultTerminal, Frame,
};
use tui_tree_widget::{Tree, TreeItem, TreeState};

#[derive(PartialEq)]
enum Focus {
    Editor,
    Tree,
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    ratatui::run(app)?;
    Ok(())
}

fn app(terminal: &mut DefaultTerminal) -> std::io::Result<()> {
    let mut focus = Focus::Tree;
    let mut editor_states: HashMap<&str, EditorState> = HashMap::new();
    let mut current_tree_key: &str = "";
    let mut tree_state: TreeState<&str> = TreeState::default();
    let mut event_handler = EditorEventHandler::default();
    let mut last_key: Option<crossterm::event::KeyCode> = None;
    let mut choose_path_toogle = false;
    let mut show_editor = false;
    loop {
        let mut state = editor_states
            .entry(current_tree_key)
            .or_insert_with(EditorState::default);

        terminal.draw(|frame| {
            render(
                frame,
                &mut state,
                &mut tree_state,
                choose_path_toogle,
                show_editor,
            )
        })?;

        let event = crossterm::event::read()?;

        if event.is_key_press() {
            if let crossterm::event::Event::Key(key_event) = event {
                if key_event.code == crossterm::event::KeyCode::Tab {
                    focus = match focus {
                        Focus::Editor => Focus::Tree,
                        Focus::Tree => Focus::Editor,
                    };
                    continue;
                }

                if key_event.modifiers == crossterm::event::KeyModifiers::CONTROL {
                    if key_event.code == crossterm::event::KeyCode::Char('q') {
                        break Ok(());
                    }
                    if key_event.code == crossterm::event::KeyCode::Char('s') {
                        let file_name = format! {"{}.md", current_tree_key};
                        fs::write(file_name, state.lines.to_string())?;
                    }
                }

                if key_event.code == crossterm::event::KeyCode::Esc {
                    last_key = None;
                    if choose_path_toogle {
                        choose_path_toogle = false;
                        last_key = None;
                        event_handler.on_key_event(key_event, &mut state);
                        terminal.draw(|frame| {
                            render(
                                frame,
                                &mut state,
                                &mut tree_state,
                                choose_path_toogle,
                                show_editor,
                            )
                        })?;
                        continue;
                    }
                }
                if focus == Focus::Editor {
                    if state.mode == edtui::EditorMode::Normal {
                        if key_event.code == crossterm::event::KeyCode::Char('f') {
                            if last_key == Some(crossterm::event::KeyCode::Char('f')) {
                                choose_path_toogle = true;
                                last_key = None;
                                terminal.draw(|frame| {
                                    render(
                                        frame,
                                        &mut state,
                                        &mut tree_state,
                                        choose_path_toogle,
                                        show_editor,
                                    )
                                })?;
                                continue;
                            } else {
                                last_key = Some(key_event.code);
                                continue;
                            }
                        } else {
                            last_key = None;
                        }
                    } else {
                        last_key = None;
                    }

                    event_handler.on_key_event(key_event, &mut state);
                } else if focus == Focus::Tree {
                    match key_event.code {
                        crossterm::event::KeyCode::Down => {
                            tree_state.key_down();
                        }
                        crossterm::event::KeyCode::Up => {
                            tree_state.key_up();
                        }
                        crossterm::event::KeyCode::Enter => {
                            tree_state.toggle_selected();
                            if let Some(selected) = tree_state.selected().first() {
                                current_tree_key = selected;
                                show_editor = true;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}

fn render(
    frame: &mut Frame,
    state: &mut EditorState,
    tree_state: &mut TreeState<&str>,
    _choose_path_toogle: bool,
    show_editor: bool,
) {
    let [left, editor_area] =
        Layout::horizontal([Constraint::Percentage(20), Constraint::Percentage(80)])
            .areas(frame.area());

    let item = TreeItem::new_leaf("l", "leaf");
    let item2 = TreeItem::new_leaf("l2", "leaf2");

    let items = vec![item, item2];

    //if choose_path_toogle {
    let tree_widget = Tree::new(&items)
        .expect("all item identifiers are unique")
        .highlight_style(Style::default().fg(Color::Black).bg(Color::White))
        .highlight_symbol(">> ")
        .block(Block::bordered().title("Tree Widget"));
    frame.render_stateful_widget(tree_widget, left, tree_state);
    //}
    if show_editor {
        let border_area = Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Red));

        let editor_inner_area = border_area.inner(editor_area);

        let syntax_highlighter = SyntaxHighlighter::new("OneHalfDark", "md");

        let theme = EditorTheme::default()
            .base(Style::default().bg(Color::Reset).fg(Color::Reset))
            .cursor_style(Style::default().bg(Color::White).fg(Color::Black))
            .line_numbers_style(Style::default().fg(Color::Gray));

        EditorView::new(state)
            .theme(theme)
            .line_numbers(LineNumbers::Absolute)
            .wrap(true)
            .syntax_highlighter(Some(syntax_highlighter.unwrap()))
            .tab_width(2)
            .render(editor_inner_area, frame.buffer_mut());
        frame.render_widget(border_area, editor_area);
    }
}
