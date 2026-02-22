#![allow(clippy::collapsible_if)]
#![allow(clippy::needless_borrow)]

use std::{collections::HashMap, fs};

use clap::Parser;
use edtui::{EditorEventHandler, EditorTheme, EditorView, LineNumbers, SyntaxHighlighter};
use ratatui::{
    layout::{Constraint, Layout},
    style::{Color, Style},
    widgets::{Block, BorderType, Widget},
    DefaultTerminal, Frame,
};
use tui_tree_widget::{Tree, TreeItem, TreeState};

mod utils;
use utils::{Args, Focus};

use crate::utils::AppState;

fn main() -> color_eyre::Result<()> {
    let args = Args::parse();

    let app_state = AppState {
        focus: Focus::Tree,
        editor_states: HashMap::new(),
        current_tree_key: String::from(""),
        tree_state: TreeState::default(),
        event_handler: EditorEventHandler::default(),
        last_key: None,
        choose_path_toogle: false,
        show_editor: false,
        preview_mode: false,
        parent_path: Some(args.path.unwrap_or_else(|| std::path::PathBuf::from("."))),
    };

    color_eyre::install()?;
    ratatui::run(|terminal| app(terminal, app_state))?;
    Ok(())
}

fn app(terminal: &mut DefaultTerminal, mut app_state: AppState) -> std::io::Result<()> {
    loop {
        terminal.draw(|frame| render(frame, &mut app_state))?;

        let event = crossterm::event::read()?;

        if event.is_key_press() {
            if let crossterm::event::Event::Key(key_event) = event {
                if key_event.code == crossterm::event::KeyCode::Tab {
                    if !app_state.show_editor {
                        app_state.focus = match app_state.focus {
                            Focus::Tree => Focus::Tree,
                            Focus::Editor => Focus::Tree,
                        };
                        continue;
                    }
                    app_state.focus = match app_state.focus {
                        Focus::Editor => Focus::Tree,
                        Focus::Tree => Focus::Editor,
                    };
                    continue;
                }

                if key_event.modifiers == crossterm::event::KeyModifiers::CONTROL {
                    if key_event.code == crossterm::event::KeyCode::Char('q') {
                        break Ok(());
                    } else if key_event.code == crossterm::event::KeyCode::Char('s') {
                        let file_name = app_state.current_tree_key.as_str();
                        let content = app_state
                            .editor_states
                            .get(&app_state.current_tree_key)
                            .map(|s| s.lines.to_string())
                            .unwrap_or_default();
                        fs::write(file_name, content)?;
                    } else if key_event.code == crossterm::event::KeyCode::Char('p') {
                        app_state.preview_mode = !app_state.preview_mode;
                    }
                }

                if key_event.code == crossterm::event::KeyCode::Esc {
                    app_state.last_key = None;
                    app_state.choose_path_toogle = false;
                }

                if app_state.focus == Focus::Editor {
                    let state = app_state
                        .editor_states
                        .entry(app_state.current_tree_key.clone())
                        .or_default();

                    if state.mode == edtui::EditorMode::Normal {
                        if key_event.code == crossterm::event::KeyCode::Char('f') {
                            if app_state.last_key == Some(crossterm::event::KeyCode::Char('f')) {
                                app_state.choose_path_toogle = true;
                                app_state.last_key = None;
                                continue;
                            } else {
                                app_state.last_key = Some(key_event.code);
                                continue;
                            }
                        } else {
                            app_state.last_key = None;
                        }
                    } else {
                        app_state.last_key = None;
                    }

                    app_state.event_handler.on_key_event(key_event, state);
                } else if app_state.focus == Focus::Tree {
                    match key_event.code {
                        crossterm::event::KeyCode::Down => {
                            app_state.tree_state.key_down();
                        }
                        crossterm::event::KeyCode::Up => {
                            app_state.tree_state.key_up();
                        }
                        crossterm::event::KeyCode::Enter => {
                            app_state.tree_state.toggle_selected();
                            if let Some(selected) = app_state.tree_state.selected().first() {
                                app_state.current_tree_key = selected.to_string();
                                app_state.show_editor = true;
                                app_state.focus = Focus::Editor;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}

fn render(frame: &mut Frame, app_state: &mut AppState) {
    let [left, editor_area] =
        Layout::horizontal([Constraint::Percentage(20), Constraint::Percentage(80)])
            .areas(frame.area());

    let items: Vec<TreeItem<String>> = app_state
        .parent_path
        .as_ref()
        .and_then(|p| fs::read_dir(p).ok())
        .map(|rd| {
            rd.filter_map(|entry| entry.ok())
                .map(|entry| {
                    let name = entry.file_name().to_string_lossy().to_string();
                    TreeItem::new_leaf(name.clone(), name)
                })
                .collect()
        })
        .unwrap_or_default();
    let tree_border_color = if app_state.focus == Focus::Tree {
        Color::Blue
    } else {
        Color::DarkGray
    };
    let editor_border_color = if app_state.focus == Focus::Editor {
        Color::Red
    } else {
        Color::DarkGray
    };

    let tree_widget = Tree::new(&items)
        .expect("all item identifiers are unique")
        .highlight_style(Style::default().fg(Color::Black).bg(Color::Blue))
        .highlight_symbol(">> ")
        .block(
            Block::bordered()
                .title("Tree Widget")
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(tree_border_color)),
        );
    frame.render_stateful_widget(tree_widget, left, &mut app_state.tree_state);

    if app_state.show_editor {
        let state = app_state
            .editor_states
            .entry(app_state.current_tree_key.clone())
            .or_default();

        if !app_state.preview_mode {
            let border_area = Block::bordered()
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(editor_border_color));
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
        } else if app_state.preview_mode {
            let content = state.lines.to_string();
            let skin = termimad::MadSkin::default();
            let rendered = skin.term_text(content.as_str()).to_string();
            let paragraph = ratatui::widgets::Paragraph::new(rendered)
                .wrap(ratatui::widgets::Wrap { trim: false });
            frame.render_widget(paragraph, editor_area);
        }
    }
}
