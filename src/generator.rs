
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

use crate::rag::Chunk;

const DEFAULT_MODEL: &str = "llama3.2:3b";
const OLLAMA_ADDRESS: &str = "127.0.0.1:11434";

/// Generate a grounded answer using the local Ollama HTTP API.
pub fn generate_answer(question: &str, hits: &[(f32, &Chunk)]) -> Result<String, String> {
    let model = std::env::var("OLLAMA_MODEL")
        .unwrap_or_else(|_| DEFAULT_MODEL.to_string());
    let prompt = build_prompt(question, hits);

    let request_body = format!(
        r#"{{"model":{},"prompt":{},"stream":false,"options":{{"num_predict":120}}}}"#,
        json_escape(&model),
        json_escape(&prompt)
    );

    let mut stream = TcpStream::connect(OLLAMA_ADDRESS).map_err(|e| {
        format!(
            "Could not connect to Ollama at {OLLAMA_ADDRESS}. \
             Make sure Ollama is running. Details: {e}"
        )
    })?;

    stream
        .set_read_timeout(Some(Duration::from_secs(300)))
        .map_err(|e| format!("Could not set Ollama read timeout: {e}"))?;

    stream
        .set_write_timeout(Some(Duration::from_secs(30)))
        .map_err(|e| format!("Could not set Ollama write timeout: {e}"))?;

    let request = format!(
        "POST /api/generate HTTP/1.1\r\n\
         Host: {OLLAMA_ADDRESS}\r\n\
         Content-Type: application/json\r\n\
         Content-Length: {}\r\n\
         Connection: close\r\n\
         \r\n\
         {}",
        request_body.len(),
        request_body
    );

    stream
        .write_all(request.as_bytes())
        .map_err(|e| format!("Could not send request to Ollama: {e}"))?;

    let mut response = Vec::new();
    stream
        .read_to_end(&mut response)
        .map_err(|e| format!("Could not read Ollama response: {e}"))?;

    let response = String::from_utf8_lossy(&response);
    let (headers, body) = response
        .split_once("\r\n\r\n")
        .ok_or_else(|| "Ollama returned an invalid HTTP response.".to_string())?;

    let status_line = headers.lines().next().unwrap_or("");
    let status_ok = status_line
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse::<u16>().ok())
        .map(|code| (200..300).contains(&code))
        .unwrap_or(false);

    if !status_ok {
        return Err(format!(
            "Ollama API returned an HTTP error: {}\n{}",
            status_line,
            body.trim()
        ));
    }

    let answer = extract_json_string(body, "response")
        .ok_or_else(|| {
            format!(
                "Ollama's response did not contain a valid answer. Response: {}",
                body.trim()
            )
        })?;

    let answer = clean_model_answer(&answer);

    if answer.is_empty() {
        return Err("The language model returned an empty answer.".to_string());
    }

    // Detect API errors accidentally returned as normal text.
    if answer.starts_with("Error: Post ")
        || answer.starts_with("Error:")
    {
        return Err(format!("Ollama generation failed: {answer}"));
    }

    Ok(answer)
}

fn build_prompt(question: &str, hits: &[(f32, &Chunk)]) -> String {
    let system = include_str!("../prompts/system_prompt.txt");
    let mut context = String::new();

    for (rank, (score, chunk)) in hits.iter().enumerate() {
        context.push_str(&format!(
            "\n[CONTEXT {} | relevance {:.3} | document: {} | section: {} | page: {}]\n{}\n",
            rank + 1,
            score,
            chunk.source,
            chunk.section,
            chunk.page
                .map(|p| p.to_string())
                .unwrap_or_else(|| "unknown".to_string()),
            chunk.text
        ));
    }

    format!(
        "{system}\n\nUSER QUESTION:\n{question}\n\n\
         RETRIEVED PROSPECTUS CONTEXT:\n{context}\n\n\
         Write the answer now. Do not mention the internal retrieval process."
    )
}

/// Encode a string as a JSON string, including quotes and escape sequences.
fn json_escape(value: &str) -> String {
    let mut output = String::from("\"");

    for ch in value.chars() {
        match ch {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            ch if ch <= '\u{001F}' => {
                output.push_str(&format!("\\u{:04x}", ch as u32));
            }
            ch => output.push(ch),
        }
    }

    output.push('"');
    output
}

/// Extract and decode a JSON string property from an API response.
fn extract_json_string(json: &str, key: &str) -> Option<String> {
    let key_marker = format!("\"{key}\"");
    let after_key = json.split_once(&key_marker)?.1;
    let after_colon = after_key.trim_start().strip_prefix(':')?.trim_start();

    let bytes = after_colon.as_bytes();

    if bytes.first() != Some(&b'"') {
        return None;
    }

    let mut result = String::new();
    let mut i = 1;

    while i < bytes.len() {
        match bytes[i] {
            b'"' => return Some(result),
            b'\\' => {
                i += 1;
                let escaped = *bytes.get(i)?;

                match escaped {
                    b'"' => result.push('"'),
                    b'\\' => result.push('\\'),
                    b'/' => result.push('/'),
                    b'n' => result.push('\n'),
                    b'r' => result.push('\r'),
                    b't' => result.push('\t'),
                    b'b' => result.push('\u{0008}'),
                    b'f' => result.push('\u{000C}'),
                    b'u' => {
                        let hex = std::str::from_utf8(
                            bytes.get(i + 1..i + 5)?
                        ).ok()?;
                        let code = u32::from_str_radix(hex, 16).ok()?;
                        result.push(char::from_u32(code)?);
                        i += 4;
                    }
                    _ => return None,
                }

                i += 1;
            }
            _ => {
                let remaining = std::str::from_utf8(&bytes[i..]).ok()?;
                let ch = remaining.chars().next()?;
                result.push(ch);
                i += ch.len_utf8();
            }
        }
    }

    None
}

fn clean_model_answer(answer: &str) -> String {
    answer
        .trim()
        .trim_start_matches("Assistant:")
        .trim()
        .to_string()
}
