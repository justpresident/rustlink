/// Terminal state for input handling and log display
#[derive(Default)]
pub struct Terminal {
    logs: Vec<String>,
    pub log_scroll: usize,
    pub input: String,
    pub cursor_pos: usize,
    pub command_history: Vec<String>,
    pub history_index: Option<usize>,
}

impl Terminal {
    pub fn new() -> Self {
        Self {
            logs: vec!["Uplink OS v0.0.1 - Hacker Edition".into()],
            ..Default::default()
        }
    }

    // Log methods
    pub fn log<S: AsRef<str>>(&mut self, message: S) {
        self.scroll_logs_to_bottom();
        self.logs.push(message.as_ref().to_string());
    }

    pub fn logs(&self) -> &[String] {
        &self.logs
    }

    pub fn logs_len(&self) -> usize {
        self.logs.len()
    }

    pub fn clear_logs(&mut self) {
        self.logs.clear();
    }

    pub fn scroll_logs_up(&mut self, amount: usize) {
        self.log_scroll = self.log_scroll.saturating_add(amount);
    }

    pub fn scroll_logs_down(&mut self, amount: usize) {
        self.log_scroll = self.log_scroll.saturating_sub(amount);
    }

    pub fn scroll_logs_to_bottom(&mut self) {
        self.log_scroll = 0;
    }

    // Input methods
    pub fn insert_char(&mut self, c: char) {
        self.input.insert(self.cursor_pos, c);
        self.cursor_pos += 1;
        self.history_index = None;
    }

    pub fn delete_char(&mut self) {
        if self.cursor_pos > 0 {
            self.cursor_pos -= 1;
            self.input.remove(self.cursor_pos);
        }
    }

    pub fn delete_char_forward(&mut self) {
        if self.cursor_pos < self.input.len() {
            self.input.remove(self.cursor_pos);
        }
    }

    pub fn move_cursor_left(&mut self) {
        if self.cursor_pos > 0 {
            self.cursor_pos -= 1;
        }
    }

    pub fn move_cursor_right(&mut self) {
        if self.cursor_pos < self.input.len() {
            self.cursor_pos += 1;
        }
    }

    pub fn move_cursor_start(&mut self) {
        self.cursor_pos = 0;
    }

    pub fn move_cursor_end(&mut self) {
        self.cursor_pos = self.input.len();
    }

    pub fn clear_line(&mut self) {
        self.input.clear();
        self.cursor_pos = 0;
    }

    pub fn delete_word(&mut self) {
        while self.cursor_pos > 0 && self.input.chars().nth(self.cursor_pos - 1) == Some(' ') {
            self.cursor_pos -= 1;
            self.input.remove(self.cursor_pos);
        }
        while self.cursor_pos > 0 && self.input.chars().nth(self.cursor_pos - 1) != Some(' ') {
            self.cursor_pos -= 1;
            self.input.remove(self.cursor_pos);
        }
    }

    // History methods
    pub fn history_up(&mut self) {
        if self.command_history.is_empty() {
            return;
        }
        match self.history_index {
            None => {
                self.history_index = Some(self.command_history.len() - 1);
            }
            Some(idx) if idx > 0 => {
                self.history_index = Some(idx - 1);
            }
            _ => return,
        }
        if let Some(idx) = self.history_index {
            self.input = self.command_history[idx].clone();
            self.cursor_pos = self.input.len();
        }
    }

    pub fn history_down(&mut self) {
        match self.history_index {
            Some(idx) if idx < self.command_history.len() - 1 => {
                self.history_index = Some(idx + 1);
                self.input = self.command_history[idx + 1].clone();
                self.cursor_pos = self.input.len();
            }
            Some(_) => {
                self.history_index = None;
                self.input.clear();
                self.cursor_pos = 0;
            }
            None => {}
        }
    }

    pub fn save_to_history(&mut self) {
        let trimmed = self.input.trim().to_string();
        if !trimmed.is_empty() && self.command_history.last() != Some(&trimmed) {
            self.command_history.push(trimmed);
        }
        self.history_index = None;
    }

    pub fn apply_completions(&mut self, completions: Vec<String>) {
        if completions.is_empty() {
            return;
        }

        if completions.len() == 1 {
            self.apply_single_completion(&completions[0]);
        } else {
            self.log(format!("Completions: {}", completions.join(" ")));
            if let Some(common) = Self::common_prefix(&completions) {
                self.apply_single_completion(&common);
            }
        }
    }

    fn apply_single_completion(&mut self, completion: &str) {
        let parts: Vec<&str> = self.input.split_whitespace().collect();
        let input_ends_with_space = self.input.ends_with(' ');

        if parts.is_empty() || (parts.len() == 1 && !input_ends_with_space) {
            self.input = completion.to_string() + " ";
        } else {
            let base_parts: Vec<&str> = if input_ends_with_space {
                parts.clone()
            } else {
                parts[..parts.len() - 1].to_vec()
            };
            self.input = base_parts.join(" ");
            if !self.input.is_empty() {
                self.input.push(' ');
            }
            self.input.push_str(completion);
            self.input.push(' ');
        }
        self.cursor_pos = self.input.len();
    }

    fn common_prefix(strings: &[String]) -> Option<String> {
        if strings.is_empty() {
            return None;
        }
        let first = &strings[0];
        let mut prefix_len = first.len();
        for s in &strings[1..] {
            prefix_len = first
                .chars()
                .zip(s.chars())
                .take_while(|(a, b)| a == b)
                .count()
                .min(prefix_len);
        }
        if prefix_len > 0 {
            Some(
                first[..first
                    .char_indices()
                    .nth(prefix_len)
                    .map(|(i, _)| i)
                    .unwrap_or(first.len())]
                    .to_string(),
            )
        } else {
            None
        }
    }
}
