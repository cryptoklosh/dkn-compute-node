use colored::Colorize;
use dkn_p2p::libp2p::request_response::ResponseChannel;
use dkn_utils::payloads::{TaskRequestPayload, TaskResponsePayload, TaskStats, TASK_RESULT_TOPIC};
use dkn_utils::DriaMessage;
use dkn_workflows::{Executor, ModelProvider, TaskWorkflow};
use eyre::{eyre, Context, Result};
use libsecp256k1::PublicKey;

use crate::workers::task::*;
use crate::DriaComputeNode;

pub struct TaskResponder;

impl super::IsResponder for TaskResponder {
    type Request = DriaMessage; // TODO: TaskRequestPayload<TaskWorkflow>;
    type Response = DriaMessage; // TODO: TaskResponsePayload;
}

impl TaskResponder {
    /// Handles the compute message for workflows.
    pub(crate) async fn prepare_worker_input(
        node: &mut DriaComputeNode,
        compute_message: &DriaMessage,
        channel: ResponseChannel<Vec<u8>>,
    ) -> Result<(TaskWorkerInput, TaskWorkerMetadata)> {
        // parse payload
        let task = compute_message
            .parse_payload::<TaskRequestPayload<TaskWorkflow>>()
            .wrap_err("could not parse workflow task")?;
        log::info!("Handling task {}", task.task_id);

        let stats = TaskStats::new().record_received_at();

        // check if deadline is past or not
        // with request-response, we dont expect this to happen much
        if chrono::Utc::now() >= task.deadline {
            return Err(eyre!(
                "Task {} is past the deadline, ignoring",
                task.task_id
            ));
        }

        // obtain public key from the payload
        // do this early to avoid unnecessary processing
        let task_public_key_bytes =
            hex::decode(&task.public_key).wrap_err("could not decode public key")?;
        let task_public_key = PublicKey::parse_slice(&task_public_key_bytes, None)?;

        // read model / provider from the task
        let model = node
            .config
            .workflows
            .get_any_matching_model(task.input.model)?;
        let model_name = model.to_string(); // get model name, we will pass it in payload
        log::info!("Using model {} for task {}", model_name, task.task_id);

        // prepare workflow executor
        let (executor, batchable) = if model.provider() == ModelProvider::Ollama {
            (
                Executor::new_at(
                    model,
                    &node.config.workflows.ollama.host,
                    node.config.workflows.ollama.port,
                ),
                false,
            )
        } else {
            (Executor::new(model), true)
        };

        // get workflow as well
        let workflow = task.input.workflow;

        let task_input = TaskWorkerInput {
            executor,
            workflow,
            task_id: task.task_id,
            stats,
            batchable,
        };

        let task_metadata = TaskWorkerMetadata {
            model_name,
            public_key: task_public_key,
            channel,
        };

        Ok((task_input, task_metadata))
    }

    /// Handles the result of a workflow task.
    pub(crate) async fn send_output(
        node: &mut DriaComputeNode,
        task_output: TaskWorkerOutput,
        task_metadata: TaskWorkerMetadata,
    ) -> Result<()> {
        let response = match task_output.result {
            Ok(result) => {
                // prepare signed and encrypted payload
                log::info!(
                    "Publishing {} result for {}",
                    "task".yellow(),
                    task_output.task_id
                );
                let payload = TaskResponsePayload::new(
                    result,
                    &task_output.task_id,
                    &task_metadata.public_key,
                    task_metadata.model_name,
                    task_output.stats.record_published_at(),
                )?;

                // convert payload to message
                let payload_str = serde_json::json!(payload).to_string();

                node.new_message(payload_str, TASK_RESULT_TOPIC)
            }
            Err(err) => {
                // use pretty display string for error logging with causes
                let err_string = format!("{:#}", err);
                log::error!("Task {} failed: {}", task_output.task_id, err_string);

                // prepare error payload
                let error_payload = TaskResponsePayload::new_error(
                    err_string,
                    task_output.task_id,
                    task_metadata.model_name,
                    task_output.stats.record_published_at(),
                );
                let error_payload_str = serde_json::json!(error_payload).to_string();

                node.new_message(error_payload_str, TASK_RESULT_TOPIC)
            }
        };

        // respond through the channel
        node.p2p
            .respond(response.into(), task_metadata.channel)
            .await?;

        Ok(())
    }
}
