const MIN_CHUNK_CHARS: usize = 4;

pub(crate) struct StreamChunker {
    buffer: String,
}

impl StreamChunker {
    pub(crate) fn new() -> Self {
        Self {
            buffer: String::new(),
        }
    }

    pub(crate) fn push(&mut self, text: &str) -> Vec<String> {
        self.buffer.push_str(text);
        self.drain_sentences()
    }

    pub(crate) fn flush(self) -> Option<String> {
        let trimmed = self.buffer.trim().to_string();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    }

    fn drain_sentences(&mut self) -> Vec<String> {
        let mut sentences = Vec::new();
        let mut search_from = 0;

        loop {
            let pos = self.find_sentence_boundary_from(search_from);
            match pos {
                Some(end) => {
                    let candidate: String = self.buffer[..end].trim().to_string();

                    if !candidate.is_empty() && candidate.chars().count() >= MIN_CHUNK_CHARS {
                        self.buffer = self.buffer[end..].to_string();
                        search_from = 0;
                        sentences.push(candidate);
                    } else {
                        // Too short — skip this boundary and look for the next one
                        search_from = end;
                    }
                }
                None => break,
            }
        }

        sentences
    }

    fn find_sentence_boundary_from(&self, from: usize) -> Option<usize> {
        for (i, ch) in self.buffer[from..].char_indices() {
            let abs_pos = from + i;
            if ch == '\n' {
                let before = self.buffer[..abs_pos].trim();
                if !before.is_empty() {
                    return Some(abs_pos + 1);
                }
            }
            if is_sentence_terminator(ch) {
                return Some(abs_pos + ch.len_utf8());
            }
        }
        None
    }
}

fn is_sentence_terminator(ch: char) -> bool {
    matches!(ch, '。' | '！' | '？' | '!' | '?')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_sentence_split() {
        let mut chunker = StreamChunker::new();
        let result = chunker.push("こんにちは。元気ですか？");
        assert_eq!(result, vec!["こんにちは。", "元気ですか？"]);
    }

    #[test]
    fn test_buffering_partial() {
        let mut chunker = StreamChunker::new();
        assert!(chunker.push("こんに").is_empty());
        assert!(chunker.push("ちは").is_empty());
        let result = chunker.push("。");
        assert_eq!(result, vec!["こんにちは。"]);
    }

    #[test]
    fn test_flush_remaining() {
        let mut chunker = StreamChunker::new();
        let _ = chunker.push("最後のテキスト");
        let remaining = chunker.flush();
        assert_eq!(remaining, Some("最後のテキスト".to_string()));
    }

    #[test]
    fn test_flush_empty() {
        let chunker = StreamChunker::new();
        assert_eq!(chunker.flush(), None);
    }

    #[test]
    fn test_newline_split() {
        let mut chunker = StreamChunker::new();
        let result = chunker.push("一行目です\n二行目です。");
        assert_eq!(result, vec!["一行目です", "二行目です。"]);
    }

    #[test]
    fn test_min_chunk_chars() {
        let mut chunker = StreamChunker::new();
        // "あ。" is only 2 chars, below MIN_CHUNK_CHARS
        let result = chunker.push("あ。続きのテキスト。");
        // "あ。" should be merged with the next sentence
        assert_eq!(result, vec!["あ。続きのテキスト。"]);
    }

    #[test]
    fn test_incremental_chunks() {
        let mut chunker = StreamChunker::new();
        assert!(chunker.push("Rust").is_empty());
        assert!(chunker.push("は素晴ら").is_empty());
        assert_eq!(chunker.push("しい。"), vec!["Rustは素晴らしい。"]);
        assert!(chunker.push("そして").is_empty());
        assert_eq!(chunker.push("安全です！"), vec!["そして安全です！"]);
        assert_eq!(chunker.flush(), None);
    }
}
