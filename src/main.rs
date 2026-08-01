#![forbid(unsafe_code)]

use portmortem_inflection_rs::{
    camelize, dasherize, humanize, ordinal, ordinalize, parameterize, pluralize, singularize,
    tableize, titleize, transliterate, underscore,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::{self, BufRead, Write};

#[derive(Debug, Deserialize)]
struct Request {
    operation: String,
    value: Value,
    #[serde(default)]
    uppercase_first_letter: Option<bool>,
    #[serde(default)]
    separator: Option<String>,
}

#[derive(Debug, Serialize)]
struct Success {
    ok: bool,
    value: String,
}

#[derive(Debug, Serialize)]
struct Failure {
    ok: bool,
    error: ErrorBody,
}

#[derive(Debug, Serialize)]
struct ErrorBody {
    code: &'static str,
    message: String,
}

#[derive(Debug)]
struct DriverError {
    code: &'static str,
    message: String,
}

impl DriverError {
    fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

fn main() -> io::Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::BufWriter::new(io::stdout().lock());

    for line in stdin.lock().lines() {
        let line = line?;
        let response = handle_line(&line);
        serde_json::to_writer(&mut stdout, &response)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
        stdout.write_all(b"\n")?;
        stdout.flush()?;
    }
    Ok(())
}

fn handle_line(line: &str) -> Value {
    let parsed = match serde_json::from_str::<Value>(line) {
        Ok(value) => value,
        Err(error) => {
            return serde_json::to_value(Failure {
                ok: false,
                error: ErrorBody {
                    code: "invalid_json",
                    message: error.to_string(),
                },
            })
            .expect("serializing a failure response cannot fail");
        }
    };
    let request = match serde_json::from_value::<Request>(parsed) {
        Ok(request) => request,
        Err(error) => {
            return serde_json::to_value(Failure {
                ok: false,
                error: ErrorBody {
                    code: "invalid_request",
                    message: error.to_string(),
                },
            })
            .expect("serializing a failure response cannot fail");
        }
    };

    match execute(&request) {
        Ok(value) => serde_json::to_value(Success { ok: true, value })
            .expect("serializing a success response cannot fail"),
        Err(error) => serde_json::to_value(Failure {
            ok: false,
            error: ErrorBody {
                code: error.code,
                message: error.message,
            },
        })
        .expect("serializing a failure response cannot fail"),
    }
}

fn execute(request: &Request) -> Result<String, DriverError> {
    match request.operation.as_str() {
        "camelize" => {
            let value = string_value(request)?;
            let uppercase_first_letter = request.uppercase_first_letter.unwrap_or(true);
            if value.is_empty() && !uppercase_first_letter {
                return Err(DriverError::new(
                    "reference_index_error",
                    "string index out of range",
                ));
            }
            Ok(camelize(value, uppercase_first_letter))
        }
        "dasherize" => Ok(dasherize(string_value(request)?)),
        "humanize" => Ok(humanize(string_value(request)?)),
        "ordinal" => Ok(ordinal(integer_value(request)?).to_owned()),
        "ordinalize" => Ok(ordinalize(integer_value(request)?)),
        "parameterize" => parameterize(
            string_value(request)?,
            request.separator.as_deref().unwrap_or("-"),
        )
        .map_err(|error| DriverError::new(error.code(), error.to_string())),
        "pluralize" => Ok(pluralize(string_value(request)?)),
        "singularize" => Ok(singularize(string_value(request)?)),
        "tableize" => Ok(tableize(string_value(request)?)),
        "titleize" => Ok(titleize(string_value(request)?)),
        "transliterate" => Ok(transliterate(string_value(request)?)),
        "underscore" => Ok(underscore(string_value(request)?)),
        operation => Err(DriverError::new(
            "unknown_operation",
            format!("unsupported operation: {operation}"),
        )),
    }
}

fn string_value(request: &Request) -> Result<&str, DriverError> {
    request.value.as_str().ok_or_else(|| {
        DriverError::new(
            "invalid_value_type",
            format!("{} requires a string value", request.operation),
        )
    })
}

fn integer_value(request: &Request) -> Result<i64, DriverError> {
    request.value.as_i64().ok_or_else(|| {
        DriverError::new(
            "invalid_value_type",
            format!(
                "{} requires a signed 64-bit integer value",
                request.operation
            ),
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn driver_uses_reference_defaults() {
        assert_eq!(
            handle_line(r#"{"operation":"camelize","value":"device_type"}"#),
            serde_json::json!({"ok": true, "value": "DeviceType"})
        );
        assert_eq!(
            handle_line(r#"{"operation":"parameterize","value":"Donald E. Knuth"}"#),
            serde_json::json!({"ok": true, "value": "donald-e-knuth"})
        );
    }

    #[test]
    fn driver_returns_structured_errors() {
        let response = handle_line(r#"{"operation":"ordinal","value":"one"}"#);
        assert_eq!(response["ok"], false);
        assert_eq!(response["error"]["code"], "invalid_value_type");

        let response = handle_line("not-json");
        assert_eq!(response["ok"], false);
        assert_eq!(response["error"]["code"], "invalid_json");

        let response =
            handle_line(r#"{"operation":"camelize","value":"","uppercase_first_letter":false}"#);
        assert_eq!(
            response,
            serde_json::json!({
                "ok": false,
                "error": {
                    "code": "reference_index_error",
                    "message": "string index out of range"
                }
            })
        );
    }
}
