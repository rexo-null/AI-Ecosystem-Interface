// Tool Use Protocol: LLM → structured JSON → schema validation → PolicyEngine → tool call → result

use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;
use std::collections::HashMap;

/// Tool call from LLM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub name: String,
    pub arguments: HashMap<String, Value>,
    pub call_id: String,
}

/// Tool execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub call_id: String,
    pub success: bool,
    pub output: Option<String>,
    pub error: Option<String>,
}

/// Tool validation error
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum ToolValidationError {
    #[error("Invalid JSON: {0}")]
    InvalidJson(String),
    #[error("Schema mismatch: {0}")]
    SchemaMismatch(String),
    #[error("Policy denied: {0}")]
    PolicyDenied(String),
    #[error("Tool not found: {0}")]
    ToolNotFound(String),
    #[error("Missing required argument: {0}")]
    MissingArgument(String),
    #[error("Invalid argument type: {0}")]
    InvalidArgumentType(String),
}

/// Tool schema definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSchema {
    pub name: String,
    pub description: String,
    pub parameters: HashMap<String, ParameterSchema>,
    pub required: Vec<String>,
    pub risk_level: RiskLevel,
}

/// Parameter schema for validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterSchema {
    pub param_type: String, // "string", "number", "boolean", "array", "object"
    pub description: String,
    pub default: Option<Value>,
    pub enum_values: Option<Vec<Value>>,
}

/// Risk level for policy checks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskLevel {
    Safe,
    Medium,
    High,
    Dangerous,
}

/// Tool Use Protocol handler
pub struct ToolUseProtocol {
    tools: HashMap<String, ToolSchema>,
}

impl ToolUseProtocol {
    /// Create a new protocol with registered tools
    pub fn new() -> Self {
        let mut tools = HashMap::new();
        
        // Register built-in tools
        tools.insert("file_read".to_string(), ToolSchema {
            name: "file_read".to_string(),
            description: "Read content of a file".to_string(),
            parameters: HashMap::from([
                ("path".to_string(), ParameterSchema {
                    param_type: "string".to_string(),
                    description: "File path to read".to_string(),
                    default: None,
                    enum_values: None,
                }),
            ]),
            required: vec!["path".to_string()],
            risk_level: RiskLevel::Safe,
        });

        tools.insert("file_write".to_string(), ToolSchema {
            name: "file_write".to_string(),
            description: "Write content to a file".to_string(),
            parameters: HashMap::from([
                ("path".to_string(), ParameterSchema {
                    param_type: "string".to_string(),
                    description: "File path to write".to_string(),
                    default: None,
                    enum_values: None,
                }),
                ("content".to_string(), ParameterSchema {
                    param_type: "string".to_string(),
                    description: "Content to write".to_string(),
                    default: None,
                    enum_values: None,
                }),
            ]),
            required: vec!["path".to_string(), "content".to_string()],
            risk_level: RiskLevel::Medium,
        });

        tools.insert("file_list".to_string(), ToolSchema {
            name: "file_list".to_string(),
            description: "List files in a directory".to_string(),
            parameters: HashMap::from([
                ("path".to_string(), ParameterSchema {
                    param_type: "string".to_string(),
                    description: "Directory path".to_string(),
                    default: Some(Value::String(".".to_string())),
                    enum_values: None,
                }),
            ]),
            required: vec![],
            risk_level: RiskLevel::Safe,
        });

        tools.insert("shell_exec".to_string(), ToolSchema {
            name: "shell_exec".to_string(),
            description: "Execute a shell command".to_string(),
            parameters: HashMap::from([
                ("command".to_string(), ParameterSchema {
                    param_type: "string".to_string(),
                    description: "Shell command to execute".to_string(),
                    default: None,
                    enum_values: None,
                }),
                ("cwd".to_string(), ParameterSchema {
                    param_type: "string".to_string(),
                    description: "Working directory".to_string(),
                    default: Some(Value::String(".".to_string())),
                    enum_values: None,
                }),
            ]),
            required: vec!["command".to_string()],
            risk_level: RiskLevel::High,
        });

        tools.insert("search_code".to_string(), ToolSchema {
            name: "search_code".to_string(),
            description: "Search code by pattern".to_string(),
            parameters: HashMap::from([
                ("pattern".to_string(), ParameterSchema {
                    param_type: "string".to_string(),
                    description: "Search pattern (regex or plain text)".to_string(),
                    default: None,
                    enum_values: None,
                }),
            ]),
            required: vec!["pattern".to_string()],
            risk_level: RiskLevel::Safe,
        });

        Self { tools }
    }

    /// Validate tool call against schema
    pub fn validate_tool_call(&self, tool_call: &ToolCall) -> Result<(), ToolValidationError> {
        // Check if tool exists
        let schema = self.tools.get(&tool_call.name)
            .ok_or_else(|| ToolValidationError::ToolNotFound(tool_call.name.clone()))?;

        // Check required arguments
        for required_arg in &schema.required {
            if !tool_call.arguments.contains_key(required_arg) {
                return Err(ToolValidationError::MissingArgument(required_arg.clone()));
            }
        }

        // Validate argument types
        for (param_name, param_schema) in &schema.parameters {
            if let Some(value) = tool_call.arguments.get(param_name) {
                if !Self::validate_type(value, &param_schema.param_type) {
                    return Err(ToolValidationError::InvalidArgumentType(
                        format!("{} should be {}", param_name, param_schema.param_type)
                    ));
                }

                // Check enum values if specified
                if let Some(enum_values) = &param_schema.enum_values {
                    if !enum_values.contains(value) {
                        return Err(ToolValidationError::SchemaMismatch(
                            format!("{} must be one of {:?}", param_name, enum_values)
                        ));
                    }
                }
            } else if let Some(default) = &param_schema.default {
                // Use default value
            }
        }

        Ok(())
    }

    /// Validate value matches expected type
    fn validate_type(value: &Value, expected_type: &str) -> bool {
        match expected_type {
            "string" => value.is_string(),
            "number" => value.is_number(),
            "boolean" => value.is_boolean(),
            "array" => value.is_array(),
            "object" => value.is_object(),
            _ => false,
        }
    }

    /// Get tool schema by name
    pub fn get_tool_schema(&self, name: &str) -> Option<&ToolSchema> {
        self.tools.get(name)
    }

    /// List all registered tools
    pub fn list_tools(&self) -> Vec<&ToolSchema> {
        self.tools.values().collect()
    }

    /// Parse and validate tool call from JSON string
    pub fn parse_and_validate(&self, json_str: &str) -> Result<ToolCall, ToolValidationError> {
        let tool_call: ToolCall = serde_json::from_str(json_str)
            .map_err(|e| ToolValidationError::InvalidJson(e.to_string()))?;
        
        self.validate_tool_call(&tool_call)?;
        Ok(tool_call)
    }
}

impl Default for ToolUseProtocol {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_tool_use_protocol_creation() {
        let protocol = ToolUseProtocol::new();
        assert!(protocol.list_tools().len() >= 5);
    }

    #[test]
    fn test_validate_valid_tool_call() {
        let protocol = ToolUseProtocol::new();
        
        let tool_call = ToolCall {
            name: "file_read".to_string(),
            arguments: HashMap::from([
                ("path".to_string(), json!("test.rs")),
            ]),
            call_id: "call_1".to_string(),
        };

        assert!(protocol.validate_tool_call(&tool_call).is_ok());
    }

    #[test]
    fn test_validate_missing_argument() {
        let protocol = ToolUseProtocol::new();
        
        let tool_call = ToolCall {
            name: "file_read".to_string(),
            arguments: HashMap::new(), // Missing "path"
            call_id: "call_1".to_string(),
        };

        let result = protocol.validate_tool_call(&tool_call);
        assert!(matches!(result, Err(ToolValidationError::MissingArgument(_))));
    }

    #[test]
    fn test_validate_invalid_type() {
        let protocol = ToolUseProtocol::new();
        
        let tool_call = ToolCall {
            name: "file_read".to_string(),
            arguments: HashMap::from([
                ("path".to_string(), json!(123)), // Should be string
            ]),
            call_id: "call_1".to_string(),
        };

        let result = protocol.validate_tool_call(&tool_call);
        assert!(matches!(result, Err(ToolValidationError::InvalidArgumentType(_))));
    }

    #[test]
    fn test_parse_and_validate() {
        let protocol = ToolUseProtocol::new();
        
        let json_str = r#"{
            "name": "file_write",
            "arguments": {"path": "test.txt", "content": "hello"},
            "call_id": "call_1"
        }"#;

        let result = protocol.parse_and_validate(json_str);
        assert!(result.is_ok());
        let tool_call = result.unwrap();
        assert_eq!(tool_call.name, "file_write");
    }

    #[test]
    fn test_parse_invalid_json() {
        let protocol = ToolUseProtocol::new();
        
        let json_str = r#"{"name": "file_read""#; // Invalid JSON
        
        let result = protocol.parse_and_validate(json_str);
        assert!(matches!(result, Err(ToolValidationError::InvalidJson(_))));
    }

    #[test]
    fn test_tool_not_found() {
        let protocol = ToolUseProtocol::new();
        
        let tool_call = ToolCall {
            name: "nonexistent_tool".to_string(),
            arguments: HashMap::new(),
            call_id: "call_1".to_string(),
        };

        let result = protocol.validate_tool_call(&tool_call);
        assert!(matches!(result, Err(ToolValidationError::ToolNotFound(_))));
    }

    #[test]
    fn test_get_tool_schema() {
        let protocol = ToolUseProtocol::new();
        
        let schema = protocol.get_tool_schema("file_read");
        assert!(schema.is_some());
        assert_eq!(schema.unwrap().name, "file_read");
    }
}
