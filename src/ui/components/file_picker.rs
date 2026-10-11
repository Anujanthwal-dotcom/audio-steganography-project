use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{List, ListItem, ListState},
    Frame,
};
use std::fs;
use std::path::PathBuf;

use crate::ui::theme::Theme;

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub is_wav: bool,
    pub size_formatted: String,
}

#[derive(Debug, Clone)]
pub struct FilePickerState {
    pub current_dir: PathBuf,
    pub entries: Vec<FileEntry>,
    pub selected_index: usize,
    pub selected_file: Option<PathBuf>,
}

impl Default for FilePickerState {
    fn default() -> Self {
        let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let mut picker = Self {
            current_dir,
            entries: Vec::new(),
            selected_index: 0,
            selected_file: None,
        };
        picker.refresh();
        picker
    }
}

impl FilePickerState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn refresh(&mut self) {
        let mut new_entries = Vec::new();

        if let Some(parent) = self.current_dir.parent() {
            new_entries.push(FileEntry {
                name: "..".to_string(),
                path: parent.to_path_buf(),
                is_dir: true,
                is_wav: false,
                size_formatted: String::new(),
            });
        }

        if let Ok(dir_entries) = fs::read_dir(&self.current_dir) {
            let mut dirs = Vec::new();
            let mut wav_files = Vec::new();
            let mut other_files = Vec::new();

            for entry in dir_entries.flatten() {
                let path = entry.path();
                let file_name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string();

                if file_name.starts_with('.') && file_name != ".." {
                    continue; // Skip hidden files
                }

                let is_dir = entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false);
                let is_wav = path
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| ext.eq_ignore_ascii_case("wav"))
                    .unwrap_or(false);

                let size_str = if !is_dir {
                    if let Ok(meta) = entry.metadata() {
                        let len = meta.len();
                        if len >= 1024 * 1024 {
                            format!("{:.1} MB", len as f64 / (1024.0 * 1024.0))
                        } else if len >= 1024 {
                            format!("{:.1} KB", len as f64 / 1024.0)
                        } else {
                            format!("{} B", len)
                        }
                    } else {
                        String::new()
                    }
                } else {
                    String::new()
                };

                let item = FileEntry {
                    name: file_name,
                    path,
                    is_dir,
                    is_wav,
                    size_formatted: size_str,
                };

                if is_dir {
                    dirs.push(item);
                } else if is_wav {
                    wav_files.push(item);
                } else {
                    other_files.push(item);
                }
            }

            dirs.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
            wav_files.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
            other_files.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

            new_entries.extend(dirs);
            new_entries.extend(wav_files);
            new_entries.extend(other_files);
        }

        self.entries = new_entries;
        if self.selected_index >= self.entries.len() && !self.entries.is_empty() {
            self.selected_index = self.entries.len() - 1;
        }
    }

    pub fn next(&mut self) {
        if !self.entries.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.entries.len();
        }
    }

    pub fn previous(&mut self) {
        if !self.entries.is_empty() {
            if self.selected_index == 0 {
                self.selected_index = self.entries.len() - 1;
            } else {
                self.selected_index -= 1;
            }
        }
    }

    /// Handles pressing Enter on the currently highlighted entry.
    /// If directory, navigates into it and returns None.
    /// If WAV file, updates selected_file and returns Some(path).
    pub fn activate_selected(&mut self) -> Option<PathBuf> {
        if let Some(entry) = self.entries.get(self.selected_index).cloned() {
            if entry.is_dir {
                self.current_dir = entry.path;
                self.selected_index = 0;
                self.refresh();
                None
            } else if entry.is_wav {
                self.selected_file = Some(entry.path.clone());
                Some(entry.path)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn set_file(&mut self, path: PathBuf) {
        if let Some(parent) = path.parent() {
            self.current_dir = parent.to_path_buf();
            self.refresh();
            if let Some(idx) = self.entries.iter().position(|e| e.path == path) {
                self.selected_index = idx;
            }
        }
        self.selected_file = Some(path);
    }
}

pub fn render_file_picker(
    frame: &mut Frame,
    area: Rect,
    picker: &FilePickerState,
    theme: &Theme,
    is_focused: bool,
    title: &str,
) {
    let block = theme.block(title, is_focused);

    let items: Vec<ListItem> = picker
        .entries
        .iter()
        .map(|entry| {
            let (icon, style) = if entry.name == ".." {
                ("📁 ", theme.muted_style())
            } else if entry.is_dir {
                ("📁 ", theme.accent_style())
            } else if entry.is_wav {
                ("🎵 ", Style::default().fg(theme.success_mint).add_modifier(Modifier::BOLD))
            } else {
                ("📄 ", theme.muted_style())
            };

            let suffix = if entry.is_dir && entry.name != ".." {
                "/".to_string()
            } else if !entry.size_formatted.is_empty() {
                format!("  ({})", entry.size_formatted)
            } else {
                String::new()
            };

            let line = Line::from(vec![
                Span::styled(icon, style),
                Span::styled(&entry.name, style),
                Span::styled(suffix, theme.muted_style()),
            ]);

            ListItem::new(line)
        })
        .collect();

    let list = List::new(items)
        .block(block)
        .highlight_style(
            Style::default()
                .bg(theme.surface)
                .fg(theme.cyan_accent)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    let mut state = ListState::default();
    if !picker.entries.is_empty() {
        state.select(Some(picker.selected_index));
    }

    frame.render_stateful_widget(list, area, &mut state);
}
