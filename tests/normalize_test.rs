use llm_voice_bridge::normalize_for_tts;

#[test]
fn test_code_block_removal() {
    let input = "前文\n```rust\nfn main() {\n    println!(\"hello\");\n}\n```\n後文";
    let result = normalize_for_tts(input);
    assert!(!result.contains("fn main"));
    assert!(!result.contains("```"));
    assert!(result.contains("前文"));
    assert!(result.contains("後文"));
}

#[test]
fn test_inline_code_removal() {
    let result = normalize_for_tts("変数`x`を使います");
    assert!(!result.contains('`'));
    assert!(result.contains("変数"));
    assert!(result.contains("を使います"));
}

#[test]
fn test_heading_removal() {
    assert_eq!(normalize_for_tts("# 見出し1"), "見出し1");
    assert_eq!(normalize_for_tts("## 見出し2"), "見出し2");
    assert_eq!(normalize_for_tts("### 見出し3"), "見出し3");
}

#[test]
fn test_bold_removal() {
    assert_eq!(normalize_for_tts("これは**太字**です"), "これは太字です");
}

#[test]
fn test_bullet_conversion() {
    let input = "- 項目A\n- 項目B";
    let result = normalize_for_tts(input);
    assert!(result.contains("項目A。"));
    assert!(result.contains("項目B。"));
}

#[test]
fn test_bullet_no_double_period() {
    let input = "- 項目A。";
    let result = normalize_for_tts(input);
    assert_eq!(result, "項目A。");
}

#[test]
fn test_url_removal() {
    let input = "詳細は https://example.com/path を参照";
    let result = normalize_for_tts(input);
    assert!(!result.contains("https://"));
    assert!(!result.contains("example.com"));
}

#[test]
fn test_empty_input() {
    assert_eq!(normalize_for_tts(""), "");
}

#[test]
fn test_plain_text_preserved() {
    let input = "これは普通のテキストです。変換は不要です。";
    assert_eq!(normalize_for_tts(input), input);
}

#[test]
fn test_multiple_code_blocks() {
    let input = "開始\n```\nblock1\n```\n中間\n```\nblock2\n```\n終了";
    let result = normalize_for_tts(input);
    assert!(!result.contains("block1"));
    assert!(!result.contains("block2"));
    assert!(result.contains("開始"));
    assert!(result.contains("中間"));
    assert!(result.contains("終了"));
}

#[test]
fn test_whitespace_normalization() {
    let input = "テスト   テスト";
    let result = normalize_for_tts(input);
    assert!(!result.contains("   "));
}

#[test]
fn test_combined_markdown() {
    let input = r#"# タイトル

**重要**な説明です。

- 手順1
- 手順2

```python
print("hello")
```

詳細は https://docs.example.com を参照してください。

`config.toml` を編集します。"#;

    let result = normalize_for_tts(input);
    assert!(!result.contains('#'));
    assert!(!result.contains("**"));
    assert!(!result.contains("```"));
    assert!(!result.contains("https://"));
    assert!(!result.contains('`'));
    assert!(result.contains("タイトル"));
    assert!(result.contains("重要"));
    assert!(result.contains("手順1。"));
}
