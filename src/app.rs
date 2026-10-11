use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::fs;
use std::path::Path;

use crate::audio::{calculate_snr, WavData};
use crate::config::Config;
use crate::stego::{
    capacity::CapacityStats, detect_stego_header, embed_message, extract_message,
    StegoHeader, TOTAL_OVERHEAD,
};
use crate::ui::components::dialogs::DialogState;
use crate::ui::components::file_picker::FilePickerState;
use crate::ui::components::text_input::TextInputState;
use crate::ui::theme::Theme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    Embed,
    Extract,
    Inspector,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmbedFocus {
    FileBrowser,
    MessageInput,
    PassphraseInput,
    CompressToggle,
    EmbedButton,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtractFocus {
    FileBrowser,
    PassphraseInput,
    UnlockButton,
    ExtractedView,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InspectorFocus {
    FileBrowser,
    Details,
}

pub struct App {
    pub mode: AppMode,
    pub running: bool,
    pub config: Config,
    pub theme: Theme,
    pub dialog: DialogState,
    pub file_picker: FilePickerState,

    // Embed mode state
    pub embed_focus: EmbedFocus,
    pub loaded_cover_wav: Option<WavData>,
    pub message_input: TextInputState,
    pub passphrase_input: TextInputState,
    pub compress_enabled: bool,
    pub last_snr: Option<f64>,

    // Extract mode state
    pub extract_focus: ExtractFocus,
    pub loaded_stego_wav: Option<WavData>,
    pub detected_header: Option<StegoHeader>,
    pub extract_passphrase_input: TextInputState,
    pub extracted_message: Option<String>,
    pub extracted_status: Option<String>,

    // Inspector mode state
    pub inspector_focus: InspectorFocus,
    pub inspected_wav: Option<WavData>,
    pub inspected_header: Option<Result<StegoHeader, String>>,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        let mut app = Self {
            mode: AppMode::Embed,
            running: true,
            config: Config::new(),
            theme: Theme::default(),
            dialog: DialogState::new(),
            file_picker: FilePickerState::new(),

            embed_focus: EmbedFocus::FileBrowser,
            loaded_cover_wav: None,
            message_input: TextInputState::new(false, true),
            passphrase_input: TextInputState::new(true, false),
            compress_enabled: true,
            last_snr: None,

            extract_focus: ExtractFocus::FileBrowser,
            loaded_stego_wav: None,
            detected_header: None,
            extract_passphrase_input: TextInputState::new(true, false),
            extracted_message: None,
            extracted_status: None,

            inspector_focus: InspectorFocus::FileBrowser,
            inspected_wav: None,
            inspected_header: None,
        };

        // If there are any wav files in current directory, auto-select the first one
        if let Some(entry) = app.file_picker.entries.iter().find(|e| e.is_wav).cloned() {
            app.load_cover_file(&entry.path);
        }

        app
    }

    pub fn load_cover_file(&mut self, path: &Path) {
        match WavData::read_from_file(path) {
            Ok(wav) => {
                self.loaded_cover_wav = Some(wav);
            }
            Err(e) => {
                self.dialog.show_error(
                    "WAV Load Error",
                    format!("Failed to parse audio file {}: {e}", path.display()),
                );
            }
        }
    }

    pub fn load_stego_file(&mut self, path: &Path) {
        match WavData::read_from_file(path) {
            Ok(wav) => {
                match detect_stego_header(&wav.samples) {
                    Ok(header) => {
                        self.detected_header = Some(header);
                    }
                    Err(_) => {
                        self.detected_header = None;
                    }
                }
                self.loaded_stego_wav = Some(wav);
                self.extracted_message = None;
                self.extracted_status = None;
            }
            Err(e) => {
                self.dialog.show_error(
                    "WAV Load Error",
                    format!("Failed to parse audio file {}: {e}", path.display()),
                );
            }
        }
    }

    pub fn inspect_file(&mut self, path: &Path) {
        match WavData::read_from_file(path) {
            Ok(wav) => {
                let scan_res = detect_stego_header(&wav.samples)
                    .map_err(|e| e.to_string());
                self.inspected_header = Some(scan_res);
                self.inspected_wav = Some(wav);
            }
            Err(e) => {
                self.dialog.show_error(
                    "WAV Load Error",
                    format!("Failed to parse audio file {}: {e}", path.display()),
                );
            }
        }
    }

    pub fn handle_key_event(&mut self, key: KeyEvent) {
        // Dismiss dialog if one is active
        if self.dialog.is_open() {
            if key.code == KeyCode::Esc || key.code == KeyCode::Enter || key.code == KeyCode::Char(' ') {
                self.dialog.dismiss();
            }
            return;
        }

        // Global exit
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            self.running = false;
            return;
        }

        // Global quick mode switching (F1-F3 or 1-3 when not focused on a text editor)
        let is_typing_message = self.mode == AppMode::Embed && self.embed_focus == EmbedFocus::MessageInput;
        let is_typing_pass = (self.mode == AppMode::Embed && self.embed_focus == EmbedFocus::PassphraseInput)
            || (self.mode == AppMode::Extract && self.extract_focus == ExtractFocus::PassphraseInput);

        let is_text_editing = is_typing_message || is_typing_pass;

        if !is_text_editing {
            match key.code {
                KeyCode::Char('q') | KeyCode::Char('Q') => {
                    self.running = false;
                    return;
                }
                KeyCode::Char('1') | KeyCode::F(1) => {
                    self.mode = AppMode::Embed;
                    return;
                }
                KeyCode::Char('2') | KeyCode::F(2) => {
                    self.mode = AppMode::Extract;
                    return;
                }
                KeyCode::Char('3') | KeyCode::F(3) => {
                    self.mode = AppMode::Inspector;
                    return;
                }
                KeyCode::Char('e') | KeyCode::Char('E') if self.mode != AppMode::Embed => {
                    self.mode = AppMode::Extract;
                    return;
                }
                KeyCode::Char('h') | KeyCode::Char('H') if self.mode != AppMode::Embed => {
                    self.mode = AppMode::Embed;
                    return;
                }
                _ => {}
            }
        }

        // Tab focus cycling
        if key.code == KeyCode::Tab {
            self.cycle_focus(false);
            return;
        }
        if key.code == KeyCode::BackTab {
            self.cycle_focus(true);
            return;
        }

        match self.mode {
            AppMode::Embed => self.handle_embed_key(key),
            AppMode::Extract => self.handle_extract_key(key),
            AppMode::Inspector => self.handle_inspector_key(key),
        }
    }

    fn cycle_focus(&mut self, backwards: bool) {
        match self.mode {
            AppMode::Embed => {
                self.embed_focus = if backwards {
                    match self.embed_focus {
                        EmbedFocus::FileBrowser => EmbedFocus::EmbedButton,
                        EmbedFocus::MessageInput => EmbedFocus::FileBrowser,
                        EmbedFocus::PassphraseInput => EmbedFocus::MessageInput,
                        EmbedFocus::CompressToggle => EmbedFocus::PassphraseInput,
                        EmbedFocus::EmbedButton => EmbedFocus::CompressToggle,
                    }
                } else {
                    match self.embed_focus {
                        EmbedFocus::FileBrowser => EmbedFocus::MessageInput,
                        EmbedFocus::MessageInput => EmbedFocus::PassphraseInput,
                        EmbedFocus::PassphraseInput => EmbedFocus::CompressToggle,
                        EmbedFocus::CompressToggle => EmbedFocus::EmbedButton,
                        EmbedFocus::EmbedButton => EmbedFocus::FileBrowser,
                    }
                };
            }
            AppMode::Extract => {
                self.extract_focus = if backwards {
                    match self.extract_focus {
                        ExtractFocus::FileBrowser => ExtractFocus::ExtractedView,
                        ExtractFocus::PassphraseInput => ExtractFocus::FileBrowser,
                        ExtractFocus::UnlockButton => ExtractFocus::PassphraseInput,
                        ExtractFocus::ExtractedView => ExtractFocus::UnlockButton,
                    }
                } else {
                    match self.extract_focus {
                        ExtractFocus::FileBrowser => ExtractFocus::PassphraseInput,
                        ExtractFocus::PassphraseInput => ExtractFocus::UnlockButton,
                        ExtractFocus::UnlockButton => ExtractFocus::ExtractedView,
                        ExtractFocus::ExtractedView => ExtractFocus::FileBrowser,
                    }
                };
            }
            AppMode::Inspector => {
                self.inspector_focus = match self.inspector_focus {
                    InspectorFocus::FileBrowser => InspectorFocus::Details,
                    InspectorFocus::Details => InspectorFocus::FileBrowser,
                };
            }
        }
    }

    fn handle_embed_key(&mut self, key: KeyEvent) {
        // Quick embed shortcut Ctrl+S
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('s') {
            self.perform_embed();
            return;
        }

        match self.embed_focus {
            EmbedFocus::FileBrowser => match key.code {
                KeyCode::Up | KeyCode::Char('k') => self.file_picker.previous(),
                KeyCode::Down | KeyCode::Char('j') => self.file_picker.next(),
                KeyCode::Enter => {
                    if let Some(path) = self.file_picker.activate_selected() {
                        self.load_cover_file(&path);
                    }
                }
                _ => {}
            },
            EmbedFocus::MessageInput => match key.code {
                KeyCode::Esc => self.embed_focus = EmbedFocus::FileBrowser,
                KeyCode::Enter => self.message_input.insert_char('\n'),
                KeyCode::Backspace => self.message_input.backspace(),
                KeyCode::Delete => self.message_input.delete(),
                KeyCode::Left => self.message_input.move_left(),
                KeyCode::Right => self.message_input.move_right(),
                KeyCode::Home => self.message_input.move_to_start(),
                KeyCode::End => self.message_input.move_to_end(),
                KeyCode::Char(c) => self.message_input.insert_char(c),
                _ => {}
            },
            EmbedFocus::PassphraseInput => match key.code {
                KeyCode::Esc => self.embed_focus = EmbedFocus::MessageInput,
                KeyCode::Enter => self.embed_focus = EmbedFocus::CompressToggle,
                KeyCode::Backspace => self.passphrase_input.backspace(),
                KeyCode::Delete => self.passphrase_input.delete(),
                KeyCode::Left => self.passphrase_input.move_left(),
                KeyCode::Right => self.passphrase_input.move_right(),
                KeyCode::Char(c) => self.passphrase_input.insert_char(c),
                _ => {}
            },
            EmbedFocus::CompressToggle => match key.code {
                KeyCode::Char(' ') | KeyCode::Enter => {
                    self.compress_enabled = !self.compress_enabled;
                }
                _ => {}
            },
            EmbedFocus::EmbedButton => {
                if key.code == KeyCode::Enter || key.code == KeyCode::Char(' ') {
                    self.perform_embed();
                }
            }
        }
    }

    fn handle_extract_key(&mut self, key: KeyEvent) {
        // Quick copy & save shortcuts in Extract mode
        if key.code == KeyCode::Char('c') || key.code == KeyCode::Char('C') {
            if self.extract_focus != ExtractFocus::PassphraseInput {
                self.copy_extracted_to_clipboard();
                return;
            }
        }
        if key.code == KeyCode::Char('s') || key.code == KeyCode::Char('S') {
            if self.extract_focus != ExtractFocus::PassphraseInput {
                self.save_extracted_to_file();
                return;
            }
        }

        match self.extract_focus {
            ExtractFocus::FileBrowser => match key.code {
                KeyCode::Up | KeyCode::Char('k') => self.file_picker.previous(),
                KeyCode::Down | KeyCode::Char('j') => self.file_picker.next(),
                KeyCode::Enter => {
                    if let Some(path) = self.file_picker.activate_selected() {
                        self.load_stego_file(&path);
                    }
                }
                _ => {}
            },
            ExtractFocus::PassphraseInput => match key.code {
                KeyCode::Esc => self.extract_focus = ExtractFocus::FileBrowser,
                KeyCode::Enter => self.perform_extract(),
                KeyCode::Backspace => self.extract_passphrase_input.backspace(),
                KeyCode::Delete => self.extract_passphrase_input.delete(),
                KeyCode::Left => self.extract_passphrase_input.move_left(),
                KeyCode::Right => self.extract_passphrase_input.move_right(),
                KeyCode::Char(c) => self.extract_passphrase_input.insert_char(c),
                _ => {}
            },
            ExtractFocus::UnlockButton => {
                if key.code == KeyCode::Enter || key.code == KeyCode::Char(' ') {
                    self.perform_extract();
                }
            }
            ExtractFocus::ExtractedView => match key.code {
                KeyCode::Esc => self.extract_focus = ExtractFocus::FileBrowser,
                _ => {}
            },
        }
    }

    fn handle_inspector_key(&mut self, key: KeyEvent) {
        match self.inspector_focus {
            InspectorFocus::FileBrowser => match key.code {
                KeyCode::Up | KeyCode::Char('k') => {
                    self.file_picker.previous();
                    if let Some(entry) = self.file_picker.entries.get(self.file_picker.selected_index) {
                        if entry.is_wav {
                            self.inspect_file(&entry.path.clone());
                        }
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.file_picker.next();
                    if let Some(entry) = self.file_picker.entries.get(self.file_picker.selected_index) {
                        if entry.is_wav {
                            self.inspect_file(&entry.path.clone());
                        }
                    }
                }
                KeyCode::Enter => {
                    if let Some(path) = self.file_picker.activate_selected() {
                        self.inspect_file(&path);
                    }
                }
                _ => {}
            },
            InspectorFocus::Details => match key.code {
                KeyCode::Esc => self.inspector_focus = InspectorFocus::FileBrowser,
                KeyCode::Char('1') => {
                    if let Some(wav) = &self.inspected_wav {
                        self.loaded_cover_wav = Some(wav.clone());
                        self.mode = AppMode::Embed;
                    }
                }
                KeyCode::Char('2') => {
                    if let Some(wav) = &self.inspected_wav {
                        if let Some(path) = &wav.metadata.file_path {
                            self.load_stego_file(&path.clone());
                            self.mode = AppMode::Extract;
                        }
                    }
                }
                _ => {}
            },
        }
    }

    pub fn perform_embed(&mut self) {
        let cover_wav = match &self.loaded_cover_wav {
            Some(w) => w,
            None => {
                self.dialog.show_warning(
                    "No Audio File",
                    "Please select a carrier .wav file from the file browser first.",
                );
                return;
            }
        };

        let message = self.message_input.content.trim();
        if message.is_empty() {
            self.dialog.show_warning(
                "Empty Message",
                "Please enter a secret message to embed in the Secret Message Editor.",
            );
            return;
        }

        let cap_stats = CapacityStats::calculate(
            cover_wav.metadata.total_samples,
            message.len(),
            TOTAL_OVERHEAD,
        );

        if !cap_stats.fits() {
            self.dialog.show_error(
                "Capacity Exceeded",
                format!(
                    "The message payload ({} bytes) exceeds the audio carrier's max capacity ({} bytes).",
                    message.len() + TOTAL_OVERHEAD,
                    cap_stats.max_capacity_bytes
                ),
            );
            return;
        }

        let passphrase = if self.passphrase_input.content.trim().is_empty() {
            None
        } else {
            Some(self.passphrase_input.content.as_str())
        };

        let mut stego_wav = cover_wav.clone();
        match embed_message(
            &mut stego_wav.samples,
            message.as_bytes(),
            passphrase,
            self.compress_enabled,
        ) {
            Ok(frame) => {
                let original_path = cover_wav
                    .metadata
                    .file_path
                    .as_deref()
                    .unwrap_or(Path::new("audio.wav"));

                let stem = original_path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("output");

                let parent = original_path.parent().unwrap_or(Path::new("."));
                let output_file_name = format!("{stem}{}", self.config.default_output_suffix);
                let output_path = parent.join(&output_file_name);

                match stego_wav.write_to_file(&output_path) {
                    Ok(()) => {
                        let snr = calculate_snr(&cover_wav.samples, &stego_wav.samples);
                        self.last_snr = Some(snr);

                        // Refresh file picker
                        self.file_picker.refresh();

                        let enc_text = if frame.header.is_encrypted() {
                            "ChaCha20-Poly1305 (Argon2id)"
                        } else {
                            "Unencrypted"
                        };

                        let comp_text = if frame.header.is_compressed() {
                            "Deflate Compressed"
                        } else {
                            "Raw"
                        };

                        self.dialog.show_success(
                            "Stego Audio Generated Successfully",
                            format!(
                                "Embedded {} bytes into {}\n\nEncryption: {}\nCompression: {}\nSignal-to-Noise Ratio (SNR): {:.2} dB (Audibly Imperceptible)\nIntegrity CRC: 0x{:08X}",
                                frame.payload.len(),
                                output_file_name,
                                enc_text,
                                comp_text,
                                snr,
                                frame.crc32
                            ),
                        );
                    }
                    Err(e) => {
                        self.dialog.show_error(
                            "Save Error",
                            format!("Failed to write output audio file {}: {e}", output_path.display()),
                        );
                    }
                }
            }
            Err(e) => {
                self.dialog.show_error("Embedding Error", format!("Failed to embed stego message: {e}"));
            }
        }
    }

    pub fn perform_extract(&mut self) {
        let stego_wav = match &self.loaded_stego_wav {
            Some(w) => w,
            None => {
                self.dialog.show_warning(
                    "No Audio File",
                    "Please select a stego .wav file from the file browser first.",
                );
                return;
            }
        };

        let passphrase = if self.extract_passphrase_input.content.is_empty() {
            None
        } else {
            Some(self.extract_passphrase_input.content.as_str())
        };

        match extract_message(&stego_wav.samples, passphrase) {
            Ok(bytes) => {
                let msg = String::from_utf8_lossy(&bytes).to_string();
                self.extracted_message = Some(msg);
                self.extracted_status = Some("[DECRYPTION SUCCESSFUL - CRC VALID]".to_string());
                self.extract_focus = ExtractFocus::ExtractedView;

                self.dialog.show_success(
                    "Extraction Successful",
                    format!(
                        "Successfully extracted {} bytes of hidden message with valid CRC32 integrity check.",
                        bytes.len()
                    ),
                );
            }
            Err(e) => {
                self.extracted_status = Some(format!("[EXTRACTION FAILED: {e}]"));
                self.dialog.show_error("Extraction Failed", format!("{e}"));
            }
        }
    }

    pub fn copy_extracted_to_clipboard(&mut self) {
        if let Some(msg) = &self.extracted_message {
            match arboard::Clipboard::new() {
                Ok(mut clipboard) => match clipboard.set_text(msg) {
                    Ok(()) => {
                        self.dialog.show_success(
                            "Clipboard",
                            "Extracted secret message copied to system clipboard!",
                        );
                    }
                    Err(e) => {
                        self.dialog.show_warning(
                            "Clipboard Error",
                            format!("Failed to copy to clipboard: {e}"),
                        );
                    }
                },
                Err(e) => {
                    self.dialog.show_warning(
                        "Clipboard Unavailable",
                        format!("Could not access system clipboard: {e}"),
                    );
                }
            }
        } else {
            self.dialog.show_warning(
                "No Content",
                "No secret message has been extracted yet to copy.",
            );
        }
    }

    pub fn save_extracted_to_file(&mut self) {
        if let Some(msg) = &self.extracted_message {
            let default_name = if let Some(wav) = &self.loaded_stego_wav {
                let stem = wav
                    .metadata
                    .file_path
                    .as_deref()
                    .and_then(|p| p.file_stem())
                    .and_then(|s| s.to_str())
                    .unwrap_or("extracted");
                format!("{stem}_extracted.txt")
            } else {
                "extracted_secret.txt".to_string()
            };

            let target_path = self.file_picker.current_dir.join(&default_name);
            match fs::write(&target_path, msg) {
                Ok(()) => {
                    self.file_picker.refresh();
                    self.dialog.show_success(
                        "File Saved",
                        format!("Saved extracted message to {}", target_path.display()),
                    );
                }
                Err(e) => {
                    self.dialog.show_error(
                        "Save Error",
                        format!("Failed to save extracted text to {}: {e}", target_path.display()),
                    );
                }
            }
        } else {
            self.dialog.show_warning(
                "No Content",
                "No secret message has been extracted yet to save.",
            );
        }
    }
}
