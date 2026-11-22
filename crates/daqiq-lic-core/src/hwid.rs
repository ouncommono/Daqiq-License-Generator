use regex::Regex;
use std::sync::OnceLock;

use crate::error::LicError;
use crate::schema::DeviceRequest;

static HWID_REGEX: OnceLock<Regex> = OnceLock::new();

fn get_hwid_regex() -> &'static Regex {
    HWID_REGEX.get_or_init(|| Regex::new(r"^fp_v1:[0-9a-f]{64}$").unwrap())
}

/// Cleans invisible Persian/Unicode characters and trims whitespace
pub fn clean_hwid_input(input: &str) -> String {
    input
        .chars()
        .filter(|&c| {
            !matches!(
                c,
                '\u{200c}' // Zero Width Non-Joiner
                | '\u{200d}' // Zero Width Joiner
                | '\u{200e}' // Left-to-Right Mark
                | '\u{200f}' // Right-to-Left Mark
                | '\u{feff}' // Byte Order Mark
                | '\u{202a}' // Left-to-Right Embedding
                | '\u{202b}' // Right-to-Left Embedding
                | '\u{202c}' // Pop Directional Formatting
                | '\u{202d}' // Left-to-Right Override
                | '\u{202e}' // Right-to-Left Override
            )
        })
        .collect::<String>()
        .trim()
        .to_lowercase()
}

/// Validates whether a single string is a strictly valid fp_v1 hardware fingerprint
pub fn validate_hwid(hwid: &str) -> Result<String, LicError> {
    let cleaned = clean_hwid_input(hwid);
    if get_hwid_regex().is_match(&cleaned) {
        Ok(cleaned)
    } else {
        Err(LicError::Hwid(format!(
            "شناسه سخت‌افزاری نامعتبر است: '{}'. فرمت صحیح: fp_v1:<64 نویسه هگز>",
            hwid.trim()
        )))
    }
}

/// Bulk parses HWIDs from arbitrary multiline or delimited text input
pub fn parse_bulk_hwids(text: &str) -> (Vec<String>, Vec<String>) {
    let mut valid = Vec::new();
    let mut invalid = Vec::new();

    // Replace commas, semicolons, tabs with newlines
    let normalized = text.replace([';', ',', '\t'], "\n");

    for line in normalized.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Strip bullet numbers like "1.", "1-", "- ", "* "
        let clean_line = if let Some(idx) = trimmed.find("fp_v1:") {
            &trimmed[idx..]
        } else {
            trimmed
        };

        // Extract the token up to the first space or end
        let token = clean_line.split_whitespace().next().unwrap_or(clean_line);
        let cleaned = clean_hwid_input(token);

        if get_hwid_regex().is_match(&cleaned) {
            if !valid.contains(&cleaned) {
                valid.push(cleaned);
            }
        } else {
            invalid.push(trimmed.to_string());
        }
    }

    (valid, invalid)
}

/// Parses a `.daqreq` JSON string into a DeviceRequest and validates its HWID
pub fn parse_daqreq(json_str: &str) -> Result<DeviceRequest, LicError> {
    let req: DeviceRequest = serde_json::from_str(json_str)
        .map_err(|e| LicError::Hwid(format!("خطا در ساختار فایل درخواست سخت‌افزار (.daqreq): {e}")))?;

    if req.request_version != 1 {
        return Err(LicError::Hwid(format!(
            "نسخه درخواست سخت‌افزار پشتیبانی نمی‌شود: {}",
            req.request_version
        )));
    }

    validate_hwid(&req.device_fingerprint)?;

    Ok(req)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_hwid() {
        let valid_fp = "fp_v1:b8782278c4c15107a8fd2a5d21718e3328c8baeac5d1a3f0664701ac2983e52e";
        assert!(validate_hwid(valid_fp).is_ok());

        // With Persian ZWNJ and spaces
        let dirty = format!(" \u{200c}{valid_fp} \u{200f} ");
        assert_eq!(validate_hwid(&dirty).unwrap(), valid_fp);
    }

    #[test]
    fn test_invalid_hwid() {
        assert!(validate_hwid("invalid_fp").is_err());
        assert!(validate_hwid("fp_v1:1234").is_err());
        assert!(validate_hwid("fp_v2:b8782278c4c15107a8fd2a5d21718e3328c8baeac5d1a3f0664701ac2983e52e").is_err());
    }

    #[test]
    fn test_bulk_parse() {
        let text = r#"
            1. fp_v1:b8782278c4c15107a8fd2a5d21718e3328c8baeac5d1a3f0664701ac2983e52e
            - fp_v1:cc12b7ba73d61e6240c106bb3239c09690102a0ca9f448497532733e8a5382cf
            something_invalid
        "#;
        let (valid, invalid) = parse_bulk_hwids(text);
        assert_eq!(valid.len(), 2);
        assert_eq!(invalid.len(), 1);
    }
}
