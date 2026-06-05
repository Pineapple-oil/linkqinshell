use linkqinshell_core::{AppError, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalBuffer {
    text: String,
    max_bytes: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchMatch {
    pub start: usize,
    pub end: usize,
}

impl TerminalBuffer {
    pub fn new(max_bytes: usize) -> Self {
        Self {
            text: String::new(),
            max_bytes,
        }
    }

    pub fn push_bytes(&mut self, bytes: &[u8]) -> Result<()> {
        let chunk = String::from_utf8_lossy(bytes);
        self.text.push_str(&strip_basic_ansi(&chunk));
        if self.text.len() > self.max_bytes {
            let keep_from = self.text.len() - self.max_bytes;
            self.text = self.text.split_off(keep_from);
        }
        Ok(())
    }

    pub fn clear(&mut self) {
        self.text.clear();
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn search(&self, query: &str, case_sensitive: bool) -> Result<Vec<SearchMatch>> {
        if query.is_empty() {
            return Err(AppError::Terminal("search query is required".into()));
        }
        let haystack = if case_sensitive {
            self.text.clone()
        } else {
            self.text.to_lowercase()
        };
        let needle = if case_sensitive {
            query.to_string()
        } else {
            query.to_lowercase()
        };

        let mut matches = Vec::new();
        let mut offset = 0;
        while let Some(found) = haystack[offset..].find(&needle) {
            let start = offset + found;
            let end = start + needle.len();
            matches.push(SearchMatch { start, end });
            offset = end;
        }
        Ok(matches)
    }
}

fn strip_basic_ansi(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\u{1b}' && chars.peek() == Some(&'[') {
            chars.next();
            for next in chars.by_ref() {
                if next.is_ascii_alphabetic() {
                    break;
                }
            }
        } else {
            output.push(ch);
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terminal_buffer_retains_text_and_strips_basic_ansi() {
        let mut buffer = TerminalBuffer::new(1024);
        buffer.push_bytes(b"\x1b[31merror\x1b[0m\n").unwrap();
        assert_eq!(buffer.text(), "error\n");
    }

    #[test]
    fn search_finds_case_insensitive_matches() {
        let mut buffer = TerminalBuffer::new(1024);
        buffer.push_bytes(b"Deploy OK\ndeploy failed").unwrap();
        let matches = buffer.search("deploy", false).unwrap();
        assert_eq!(matches.len(), 2);
    }
}
