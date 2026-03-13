/// Normalize text for TTS by removing Markdown formatting and cleaning up whitespace.
pub fn normalize_for_tts(text: &str) -> String {
    let mut lines: Vec<String> = Vec::new();
    let mut in_code_block = false;

    for line in text.lines() {
        let trimmed = line.trim();

        // Toggle code block state
        if trimmed.starts_with("```") {
            in_code_block = !in_code_block;
            continue;
        }

        // Skip lines inside code blocks
        if in_code_block {
            continue;
        }

        let mut processed = line.to_string();

        // Remove inline code
        processed = remove_inline_code(&processed);

        // Remove Markdown headings
        processed = remove_headings(&processed);

        // Remove bold/italic markers
        processed = remove_emphasis(&processed);

        // Convert bullet points
        processed = convert_bullets(&processed);

        // Remove URLs
        processed = remove_urls(&processed);

        lines.push(processed);
    }

    // Join and normalize whitespace
    let result = lines.join("\n");
    normalize_whitespace(&result)
}

fn remove_inline_code(text: &str) -> String {
    let mut result = String::new();
    let mut in_code = false;

    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '`' {
            in_code = !in_code;
        } else if !in_code {
            result.push(ch);
        }
    }
    result
}

fn remove_headings(line: &str) -> String {
    let trimmed = line.trim_start();
    if trimmed.starts_with("# ")
        || trimmed.starts_with("## ")
        || trimmed.starts_with("### ")
        || trimmed.starts_with("#### ")
        || trimmed.starts_with("##### ")
        || trimmed.starts_with("###### ")
    {
        let content = trimmed.trim_start_matches('#').trim_start();
        return content.to_string();
    }
    line.to_string()
}

fn remove_emphasis(text: &str) -> String {
    let mut result = text.to_string();
    // Remove bold markers (** and __)
    while result.contains("**") {
        result = result.replacen("**", "", 1);
    }
    while result.contains("__") {
        result = result.replacen("__", "", 1);
    }
    // Remove remaining italic markers (* and _)
    // Be careful with _ in words - only remove if surrounded by whitespace or at boundaries
    result = remove_marker(&result, '*');
    result
}

fn remove_marker(text: &str, marker: char) -> String {
    let mut result = String::new();
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();

    let mut i = 0;
    while i < len {
        if chars[i] == marker {
            // Check if it's an emphasis marker (at word boundary)
            let prev_space = i == 0 || chars[i - 1].is_whitespace();
            let next_exists = i + 1 < len && !chars[i + 1].is_whitespace();

            if prev_space && next_exists {
                // Opening marker - skip it
                i += 1;
                continue;
            }

            let prev_exists = i > 0 && !chars[i - 1].is_whitespace();
            let next_space = i + 1 >= len || chars[i + 1].is_whitespace();

            if prev_exists && next_space {
                // Closing marker - skip it
                i += 1;
                continue;
            }
        }
        result.push(chars[i]);
        i += 1;
    }
    result
}

fn convert_bullets(line: &str) -> String {
    let trimmed = line.trim_start();
    for prefix in &["- ", "* "] {
        if let Some(content) = trimmed.strip_prefix(prefix) {
            if !content.is_empty() {
                return format!("{}。", content.trim_end_matches('。'));
            }
        }
    }
    line.to_string()
}

fn remove_urls(text: &str) -> String {
    let mut result = String::new();
    let mut remaining = text;

    while !remaining.is_empty() {
        if remaining.starts_with("https://") || remaining.starts_with("http://") {
            // Skip until whitespace or end
            match remaining.find(char::is_whitespace) {
                Some(pos) => remaining = &remaining[pos..],
                None => break,
            }
        } else {
            let ch = remaining.chars().next().unwrap();
            result.push(ch);
            remaining = &remaining[ch.len_utf8()..];
        }
    }
    result
}

fn normalize_whitespace(text: &str) -> String {
    let mut result = String::new();
    let mut prev_was_space = false;
    let mut prev_was_newline = false;

    for ch in text.chars() {
        if ch == '\n' {
            if !prev_was_newline {
                result.push('\n');
            }
            prev_was_newline = true;
            prev_was_space = false;
        } else if ch.is_whitespace() {
            if !prev_was_space && !prev_was_newline {
                result.push(' ');
            }
            prev_was_space = true;
        } else {
            result.push(ch);
            prev_was_space = false;
            prev_was_newline = false;
        }
    }

    result.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_code_blocks() {
        let input = "Hello\n```rust\nfn main() {}\n```\nWorld";
        assert_eq!(normalize_for_tts(input), "Hello\nWorld");
    }

    #[test]
    fn test_remove_inline_code() {
        let input = "Use `println!` to print";
        assert_eq!(normalize_for_tts(input), "Use to print");
    }

    #[test]
    fn test_remove_headings() {
        assert_eq!(normalize_for_tts("# Title"), "Title");
        assert_eq!(normalize_for_tts("## Subtitle"), "Subtitle");
        assert_eq!(normalize_for_tts("### Deep"), "Deep");
    }

    #[test]
    fn test_remove_bold() {
        assert_eq!(normalize_for_tts("This is **bold** text"), "This is bold text");
    }

    #[test]
    fn test_convert_bullets() {
        assert_eq!(normalize_for_tts("- item one"), "item one。");
        assert_eq!(normalize_for_tts("* item two"), "item two。");
    }

    #[test]
    fn test_remove_urls() {
        let input = "Visit https://example.com for more";
        assert_eq!(normalize_for_tts(input), "Visit for more");
    }

    #[test]
    fn test_empty_string() {
        assert_eq!(normalize_for_tts(""), "");
    }

    #[test]
    fn test_plain_text_unchanged() {
        let input = "普通のテキストです";
        assert_eq!(normalize_for_tts(input), "普通のテキストです");
    }

    #[test]
    fn test_combined() {
        let input = "# タイトル\n\n**太字**のテキスト\n\n- 項目1\n- 項目2\n\n```\ncode\n```\n\n終わり";
        let result = normalize_for_tts(input);
        assert!(result.contains("タイトル"));
        assert!(!result.contains("#"));
        assert!(!result.contains("**"));
        assert!(!result.contains("```"));
        assert!(result.contains("終わり"));
    }
}
