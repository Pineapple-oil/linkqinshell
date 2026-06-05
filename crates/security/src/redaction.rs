const SECRET_KEYS: &[&str] = &[
    "password",
    "passwd",
    "passphrase",
    "token",
    "private_key",
    "secret",
    "authorization",
];

pub fn redact_sensitive(input: &str) -> String {
    let mut output = input.to_string();
    for key in SECRET_KEYS {
        output = redact_key_value(&output, key);
    }
    output
}

fn redact_key_value(input: &str, key: &str) -> String {
    let mut result = String::with_capacity(input.len());
    for line in input.lines() {
        let lower = line.to_ascii_lowercase();
        if let Some(index) = lower.find(key) {
            let after_key = &line[index + key.len()..];
            if after_key.trim_start().starts_with('=') || after_key.trim_start().starts_with(':') {
                let prefix_end = line.find('=').or_else(|| line.find(':')).map(|i| i + 1);
                if let Some(prefix_end) = prefix_end {
                    result.push_str(&line[..prefix_end]);
                    result.push_str(" [REDACTED]");
                    result.push('\n');
                    continue;
                }
            }
        }
        result.push_str(line);
        result.push('\n');
    }
    result.trim_end_matches('\n').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacts_password_and_token_values() {
        let redacted =
            redact_sensitive("user=root\npassword = hunter2\ntoken: abc123\nhost=example.com");
        assert!(redacted.contains("password = [REDACTED]"));
        assert!(redacted.contains("token: [REDACTED]"));
        assert!(!redacted.contains("hunter2"));
        assert!(!redacted.contains("abc123"));
        assert!(redacted.contains("host=example.com"));
    }
}
