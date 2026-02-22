use clap::Parser;
use edtui::{EditorEventHandler, EditorState};
use tui_tree_widget::TreeState;

use std::collections::HashMap;

#[derive(PartialEq)]
pub enum Focus {
    Editor,
    Tree,
}

#[derive(Parser)]
pub struct Args {
    pub path: Option<std::path::PathBuf>,
}

pub struct AppState {
    pub focus: Focus,
    pub editor_states: HashMap<String, EditorState>,
    pub current_tree_key: String,
    pub tree_state: TreeState<String>,
    pub event_handler: EditorEventHandler,
    pub last_key: Option<crossterm::event::KeyCode>,
    pub choose_path_toogle: bool,
    pub show_editor: bool,
    pub preview_mode: bool,
    pub parent_path: Option<std::path::PathBuf>,
}
