use bytes::Bytes;
use futures::stream::{BoxStream, StreamExt};
use futures::Stream;

/// Parse a byte stream of SSE into individual `data:` payloads.
/// Filters out `[DONE]` markers and empty data lines.
pub(crate) fn parse_sse_stream(
    byte_stream: impl Stream<Item = Result<Bytes, reqwest::Error>> + Send + Unpin + 'static,
) -> BoxStream<'static, String> {
    let lines_stream = futures::stream::unfold(
        (byte_stream, String::new()),
        |(mut stream, mut leftover)| async move {
            loop {
                // Try to extract a complete line from leftover
                if let Some(newline_pos) = leftover.find('\n') {
                    let line = leftover[..newline_pos].to_string();
                    leftover = leftover[newline_pos + 1..].to_string();
                    return Some((line, (stream, leftover)));
                }

                // Need more data
                use futures::StreamExt;
                match stream.next().await {
                    Some(Ok(chunk)) => {
                        leftover.push_str(&String::from_utf8_lossy(&chunk));
                    }
                    _ => {
                        // Stream ended
                        if !leftover.is_empty() {
                            let remaining = std::mem::take(&mut leftover);
                            return Some((remaining, (stream, leftover)));
                        }
                        return None;
                    }
                }
            }
        },
    );

    lines_stream
        .filter_map(|line| async move {
            let line = line.trim().to_string();
            if let Some(data) = line.strip_prefix("data: ") {
                let data = data.trim();
                if data == "[DONE]" {
                    return None;
                }
                if data.is_empty() {
                    return None;
                }
                Some(data.to_string())
            } else {
                None
            }
        })
        .boxed()
}
