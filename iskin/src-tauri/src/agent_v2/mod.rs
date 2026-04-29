// ISKIN Agent: State Machine, Tool Use Protocol, Context Compressor, and Agent Loop
// Full Devin-level autonomous agent implementation

pub mod state_machine;
pub mod tool_protocol;
pub mod context_compressor;
pub mod agent_loop;

pub use state_machine::{AgentPhase, AgentState, StateTransitionError};
pub use tool_protocol::{ToolCall, ToolResult, ToolUseProtocol, ToolSchema, ToolValidationError};
pub use context_compressor::{ContextCompressor, CompressionSummary};
pub use agent_loop::{AgentLoop, AgentConfig, AgentEvent};
