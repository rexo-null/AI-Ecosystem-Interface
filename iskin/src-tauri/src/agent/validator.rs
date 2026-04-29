use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use anyhow::Result;

use super::planner::{TaskStep, Language};

/// Validation error types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationErrorType {
    SyntaxError {
        language: Language,
        line: Option<usize>,
    },
    CompilationError {
        output: String,
    },
    TestFailed {
        output: String,
    },
    Timeout,
    PermissionDenied,
    Unknown,
}

/// Validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationResult {
    Ok,
    Incomplete {
        partial_content: String,
        continuation_hint: String,
    },
    Error {
        message: String,
        recoverable: bool,
        error_type: ValidationErrorType,
    },
}

/// Validator for task step results
pub struct Validator {
    workspace_root: PathBuf,
}

impl Validator {
    /// Create a new validator
    pub fn new(workspace_root: PathBuf) -> Self {
        Self { workspace_root }
    }

    /// Check code completeness (detect truncation)
    pub fn check_completeness(response: &str, _step: &TaskStep) -> ValidationResult {
        // Check for unbalanced brackets in code
        let mut braces = 0;
        let mut brackets = 0;
        let mut parens = 0;
        let mut in_string = false;
        let mut escape = false;

        for ch in response.chars() {
            if escape {
                escape = false;
                continue;
            }
            if ch == '\\' {
                escape = true;
                continue;
            }
            if ch == '"' || ch == '\'' {
                in_string = !in_string;
                continue;
            }
            if in_string {
                continue;
            }

            match ch {
                '{' => braces += 1,
                '}' => braces -= 1,
                '[' => brackets += 1,
                ']' => brackets -= 1,
                '(' => parens += 1,
                ')' => parens -= 1,
                _ => {}
            }
        }

        if braces < 0 || brackets < 0 || parens < 0 {
            return ValidationResult::Error {
                message: "Unbalanced closing bracket detected".into(),
                recoverable: true,
                error_type: ValidationErrorType::SyntaxError {
                    language: Language::Rust,
                    line: None,
                },
            };
        }

        if braces > 0 || brackets > 0 || parens > 0 {
            return ValidationResult::Incomplete {
                partial_content: response.to_string(),
                continuation_hint: format!(
                    "Code block is incomplete: {} unclosed braces, {} brackets, {} parens. Continue from last line.",
                    braces, brackets, parens
                ),
            };
        }

        ValidationResult::Ok
    }

    /// Check syntax for given language
    pub fn check_syntax(code: &str, language: Language) -> ValidationResult {
        match language {
            Language::Rust => Self::check_rust_syntax(code),
            Language::Json => Self::check_json_syntax(code),
            _ => ValidationResult::Ok, // Placeholder for other languages
        }
    }

    /// Check Rust syntax (basic)
    fn check_rust_syntax(code: &str) -> ValidationResult {
        // Basic checks: balanced braces, semicolons, etc.
        let mut braces = 0;
        let mut in_string = false;
        let mut escape = false;

        for ch in code.chars() {
            if escape {
                escape = false;
                continue;
            }
            if ch == '\\' {
                escape = true;
                continue;
            }
            if ch == '"' {
                in_string = !in_string;
                continue;
            }
            if in_string {
                continue;
            }

            match ch {
                '{' => braces += 1,
                '}' => braces -= 1,
                _ => {}
            }
        }

        if braces != 0 {
            return ValidationResult::Error {
                message: "Unbalanced braces in Rust code".into(),
                recoverable: true,
                error_type: ValidationErrorType::SyntaxError {
                    language: Language::Rust,
                    line: None,
                },
            };
        }

        ValidationResult::Ok
    }

    /// Check JSON syntax
    fn check_json_syntax(code: &str) -> ValidationResult {
        match serde_json::from_str::<serde_json::Value>(code) {
            Ok(_) => ValidationResult::Ok,
            Err(e) => ValidationResult::Error {
                message: format!("Invalid JSON: {}", e),
                recoverable: true,
                error_type: ValidationErrorType::SyntaxError {
                    language: Language::Json,
                    line: None,
                },
            },
        }
    }

    /// Check compilation (Rust)
    pub async fn check_compilation(
        file_path: &str,
        cwd: Option<&str>,
    ) -> ValidationResult {
        use tokio::process::Command;

        let cwd = cwd.unwrap_or(".");
        let output = Command::new("cargo")
            .args(&["check", "--manifest-path", &format!("{}/Cargo.toml", cwd)])
            .output()
            .await;

        match output {
            Ok(result) => {
                if result.status.success() {
                    ValidationResult::Ok
                } else {
                    let stderr = String::from_utf8_lossy(&result.stderr);
                    ValidationResult::Error {
                        message: format!("Compilation failed: {}", stderr),
                        recoverable: true,
                        error_type: ValidationErrorType::CompilationError {
                            output: stderr.to_string(),
                        },
                    }
                }
            }
            Err(e) => ValidationResult::Error {
                message: format!("Failed to run cargo check: {}", e),
                recoverable: false,
                error_type: ValidationErrorType::Unknown,
            },
        }
    }

    /// Main validation entry point
    pub async fn validate(
        &self,
        step: &TaskStep,
        response: &str,
    ) -> ValidationResult {
        // First check completeness
        let completeness = Self::check_completeness(response, step);
        if !matches!(completeness, ValidationResult::Ok) {
            return completeness;
        }

        // Then check syntax based on step type
        match step {
            TaskStep::GenerateCode { file_path, .. } => {
                let language = Self::detect_language_from_path(file_path);
                Self::check_syntax(response, language)
            }
            _ => ValidationResult::Ok,
        }
    }

    /// Detect language from file extension
    fn detect_language_from_path(path: &str) -> Language {
        if path.ends_with(".rs") {
            Language::Rust
        } else if path.ends_with(".json") {
            Language::Json
        } else if path.ends_with(".toml") {
            Language::Toml
        } else if path.ends_with(".yaml") || path.ends_with(".yml") {
            Language::Yaml
        } else if path.ends_with(".md") {
            Language::Markdown
        } else if path.ends_with(".sh") {
            Language::Shell
        } else if path.ends_with(".py") {
            Language::Python
        } else if path.ends_with(".ts") || path.ends_with(".tsx") {
            Language::TypeScript
        } else {
            Language::Rust // Default
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_completeness_empty() {
        let result = Validator::check_completeness("", &TaskStep::GenerateCode {
            id: uuid::Uuid::new_v4(),
            file_path: "test.rs".to_string(),
            description: "test".to_string(),
            start_line: None,
            end_line: None,
        });

        match result {
            ValidationResult::Incomplete { .. } => {}
            _ => panic!("Expected Incomplete"),
        }
    }

    #[test]
    fn test_check_completeness_valid() {
        let result = Validator::check_completeness("fn main() {}", &TaskStep::GenerateCode {
            id: uuid::Uuid::new_v4(),
            file_path: "test.rs".to_string(),
            description: "test".to_string(),
            start_line: None,
            end_line: None,
        });

        match result {
            ValidationResult::Ok => {}
            _ => panic!("Expected Ok"),
        }
    }

    #[test]
    fn test_check_completeness_unbalanced() {
        let result = Validator::check_completeness("fn main() {", &TaskStep::GenerateCode {
            id: uuid::Uuid::new_v4(),
            file_path: "test.rs".to_string(),
            description: "test".to_string(),
            start_line: None,
            end_line: None,
        });

        match result {
            ValidationResult::Incomplete { .. } => {}
            _ => panic!("Expected Incomplete"),
        }
    }

    #[test]
    fn test_check_rust_syntax_valid() {
        let result = Validator::check_syntax("fn main() { println!(\"Hello\"); }", Language::Rust);
        match result {
            ValidationResult::Ok => {}
            _ => panic!("Expected Ok"),
        }
    }

    #[test]
    fn test_check_rust_syntax_invalid() {
        let result = Validator::check_syntax("fn main() { println!(\"Hello\"); ", Language::Rust);
        match result {
            ValidationResult::Error { .. } => {}
            _ => panic!("Expected Error"),
        }
    }

    #[test]
    fn test_check_json_syntax_valid() {
        let result = Validator::check_syntax("{\"key\": \"value\"}", Language::Json);
        match result {
            ValidationResult::Ok => {}
            _ => panic!("Expected Ok"),
        }
    }

    #[test]
    fn test_check_json_syntax_invalid() {
        let result = Validator::check_syntax("{\"key\": \"value\"", Language::Json);
        match result {
            ValidationResult::Error { .. } => {}
            _ => panic!("Expected Error"),
        }
    }

    #[test]
    fn test_detect_language() {
        let validator = Validator::new(std::path::PathBuf::from("."));
        assert_eq!(validator.detect_language_from_path("test.rs"), Language::Rust);
        assert_eq!(validator.detect_language_from_path("test.json"), Language::Json);
        assert_eq!(validator.detect_language_from_path("test.py"), Language::Python);
    }
}

