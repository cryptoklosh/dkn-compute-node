use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Topic used within [`crate::DriaMessage`] for task request messages.
pub const TASK_REQUEST_TOPIC: &str = "task";

/// Topic used within [`crate::DriaMessage`] for task result messages.
pub const TASK_RESULT_TOPIC: &str = "results";

/// A computation task is the task of computing a result from a given input.
///
/// `result` and `error` are mutually-exclusive, only one of them can be `Some`:
/// - if `result` is `Some`, then it contains the result.
/// - if `error` is `Some`, then it contains the error message.
///
/// Each task belongs to a file (uniquely identified by `file_id`), and has a unique identifier (`row_id`).
/// THe `task_id` is a custom identifier given by a user.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskResponsePayload {
    /// The file that this task is associated with.
    pub file_id: Uuid,
    /// The unique identifier of the task.
    pub row_id: Uuid,
    /// The custom identifier of the task, not necessarily unique.
    pub task_id: String,
    /// Name of the model used for this task.
    pub model: String,
    /// Stats about the task execution.
    pub stats: TaskStats,
    /// Result from the LLM, as-is.
    ///
    /// If this is `None`, the task failed, and you should check the `error` field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<String>,
    /// An error, if any.
    ///
    /// If this is `Some`, you can ignore the `result` field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<TaskError>,
}

/// A generic task request, given by Dria.
///
/// Each task belongs to a file (uniquely identified by `file_id`), and has a unique identifier (`row_id`).
/// THe `task_id` is a custom identifier given by a user.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskRequestPayload<T> {
    /// The file that this task is associated with.
    pub file_id: Uuid,
    /// The unique identifier of the task.
    pub row_id: Uuid,
    /// The custom identifier of the task, not necessarily unique.
    pub task_id: String,
    /// The input to the compute function.
    pub input: T,
}

#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
pub enum TaskError {
    /// A parse error occurred while parsing the task request or response.
    #[error("Parse error: {0}")]
    ParseError(String),
    /// An error returned from the model provider.
    #[error("{provider} error ({code}): {message}")]
    ProviderError {
        /// Not necessarily an HTTP status code, but a code that the provider uses to identify the error.
        ///
        /// For example, OpenAI uses a string code like "invalid_request_error".
        code: String,
        /// The error message returned by the provider.
        ///
        /// May contain additional information about the error.
        message: String,
        /// The source of the error.
        ///
        /// Can be a provider name, or RPC etc.
        provider: String,
    },
    /// A network-related error from the client.
    #[error("HTTP error: {0}")]
    /// This is a generic HTTP error, not necessarily related to the provider.
    HttpError(String),
    /// Any other executor error that is not a provider error.
    #[error("Executor error: {0}")]
    ExecutorError(String),
    /// The task request had failed for some network reason.
    #[error("Outbound request error: {code} - {message}")]
    OutboundRequestError {
        code: String,
        /// The error message returned by the network.
        message: String,
    },
    /// Any other error
    #[error("Other error: {0}")]
    Other(String),
}

/// Task stats for diagnostics.
///
/// Returning this as the payload helps to debug the errors received at client side, and latencies.
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskStats {
    /// Timestamp at which the task was received from network & parsed.
    pub received_at: chrono::DateTime<chrono::Utc>,
    /// Timestamp at which the task was published back to network.
    pub published_at: chrono::DateTime<chrono::Utc>,
    /// Timestamp at which the task execution had started.
    pub execution_started_at: chrono::DateTime<chrono::Utc>,
    /// Timestamp at which the task execution had finished.
    pub execution_ended_at: chrono::DateTime<chrono::Utc>,
    /// Number of tokens of the result.
    pub token_count: usize,
}

impl TaskStats {
    pub fn new() -> Self {
        Self::default()
    }

    /// Records the current timestamp within `received_at`.
    pub fn record_received_at(mut self) -> Self {
        self.received_at = chrono::Utc::now();
        self
    }

    /// Records the current timestamp within `published_at`.
    pub fn record_published_at(mut self) -> Self {
        self.published_at = chrono::Utc::now();
        self
    }

    /// Records the execution start time within `execution_started_at`.
    pub fn record_execution_started_at(mut self) -> Self {
        self.execution_started_at = chrono::Utc::now();
        self
    }

    /// Records the execution end time within `execution_ended_time`.
    pub fn record_execution_ended_at(mut self) -> Self {
        self.execution_ended_at = chrono::Utc::now();
        self
    }

    /// Records the token count within `token_count`.
    pub fn record_token_count(mut self, token_count: usize) -> Self {
        self.token_count = token_count;
        self
    }
}
