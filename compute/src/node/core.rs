use colored::Colorize;
use dkn_p2p::libp2p::{Multiaddr, PeerId};
use dkn_utils::{
    payloads::{HEARTBEAT_TOPIC, SPECS_TOPIC},
    DriaMessage,
};
use eyre::{eyre, Result};
use std::time::Duration;
use tokio_util::sync::CancellationToken;

use crate::{reqres::HeartbeatRequester, DriaComputeNode};

impl DriaComputeNode {
    /// Runs the main loop of the compute node.
    /// This method is not expected to return until cancellation occurs for the given token.
    pub async fn run(&mut self, cancellation: CancellationToken) {
        // initialize the points client
        self.points_client.initialize().await;

        /// Duration between refreshing for diagnostic prints.
        const DIAGNOSTIC_REFRESH_INTERVAL_SECS: Duration = Duration::from_secs(45);
        /// Duration between refreshing for points update.
        const POINTS_REFRESH_INTERVAL_SECS: Duration = Duration::from_secs(180);
        /// Duration between refreshing the available nodes.
        const RPC_LIVENESS_REFRESH_INTERVAL_SECS: Duration = Duration::from_secs(2 * 60);
        /// Duration between each specs update sent to the RPC.
        const SPECS_INTERVAL_SECS: Duration = Duration::from_secs(60 * 5);

        let mut diagnostic_refresh_interval =
            tokio::time::interval(DIAGNOSTIC_REFRESH_INTERVAL_SECS);
        diagnostic_refresh_interval.tick().await; // move each one tick
        let mut rpc_liveness_refresh_interval =
            tokio::time::interval(RPC_LIVENESS_REFRESH_INTERVAL_SECS);
        rpc_liveness_refresh_interval.tick().await; // move each one tick

        // tick the first time a bit earlier
        let mut points_refresh_interval = tokio::time::interval(POINTS_REFRESH_INTERVAL_SECS);
        points_refresh_interval.tick().await;
        points_refresh_interval.reset_after(POINTS_REFRESH_INTERVAL_SECS / 12);

        // move one tick, and wait at least a third of the diagnostics
        let mut heartbeat_interval = tokio::time::interval(HeartbeatRequester::HEARTBEAT_DEADLINE);
        heartbeat_interval.tick().await;
        heartbeat_interval.reset_after(DIAGNOSTIC_REFRESH_INTERVAL_SECS / 3);

        // move one tick, and wait a little bit
        let mut specs_interval = tokio::time::interval(SPECS_INTERVAL_SECS);
        specs_interval.tick().await;
        specs_interval.reset_after(DIAGNOSTIC_REFRESH_INTERVAL_SECS / 6);

        loop {
            tokio::select! {
                // a task is completed by the worker & should be responded to the requesting peer
                task_response_msg_opt = self.task_output_rx.recv() => {
                    if let Some(task_response_msg) = task_response_msg_opt {
                        if let Err(e) = self.send_task_output(task_response_msg).await {
                            log::error!("Error responding to task: {:?}", e);
                        }
                    } else {
                        log::error!("task_output_rx channel closed unexpectedly, we still have {} batch and {} single tasks.", self.pending_tasks_batch.len(), self.pending_tasks_single.len());
                        break;
                    }
                },

                // a Request or Response is received by the p2p client
                reqres_msg_opt = self.reqres_rx.recv() => {
                  if let Some((peer_id, message)) = reqres_msg_opt {
                    self.handle_reqres(peer_id, message).await;
                  } else {
                    log::error!("reqres_rx channel closed unexpectedly.");
                    break;
                  }
                },

                // check peer count every now and then
                _ = diagnostic_refresh_interval.tick() => self.handle_diagnostic_refresh().await,

                // check RPC, and get a new one if we are disconnected
                _ = rpc_liveness_refresh_interval.tick() => self.handle_rpc_liveness_check().await,

                // log points every now and then
                _ = points_refresh_interval.tick() => self.handle_points_refresh().await,

                // send a heartbeat request to publish liveness info
                _ = heartbeat_interval.tick() => {
                  if let Err(e) = self.send_heartbeat().await {
                    log::error!("Error making {}: {:?}", HEARTBEAT_TOPIC.blue(), e);
                  }
                },

                // send specs to the RPC
                _ = specs_interval.tick() => {
                  if let Err(e) = self.send_specs().await {
                    log::error!("Error sending {}: {:?}", SPECS_TOPIC.green(), e);
                  }
                },

                // check if the cancellation token is cancelled
                // this is expected to be cancelled by the main thread with signal handling
                _ = cancellation.cancelled() => {
                    log::info!("Cancellation received, shutting down the node.");
                    break;
                },
            }
        }

        // print one final diagnostic as a summary
        self.handle_diagnostic_refresh().await;

        // shutdown channels
        if let Err(e) = self.shutdown().await {
            log::error!("Could not shutdown the node gracefully: {:?}", e);
        }
    }

    /// Shorthand method to create a signed message with the given data and topic.
    ///
    /// Topic was previously used for GossipSub, but kept for verbosity.
    #[inline(always)]
    pub fn new_message(&self, data: impl AsRef<[u8]>, topic: impl ToString) -> DriaMessage {
        DriaMessage::new_signed(
            data,
            topic,
            self.p2p.protocol().name.clone(),
            &self.config.secret_key,
            self.config.version,
        )
    }

    /// Dial the given peer at the given address.
    pub async fn dial_with_timeout(&mut self, peer_id: PeerId, addr: Multiaddr) -> Result<()> {
        // while not yet known, some people get stuck during the dialling step,
        // this timeout prevents that.
        const DIAL_TIMEOUT: Duration = Duration::from_secs(10);

        match tokio::time::timeout(DIAL_TIMEOUT, self.p2p.dial(peer_id, addr)).await {
            Err(timeout) => Err(eyre!("Timeout dialling RPC node: {}", timeout)),
            Ok(result) => result, // this is also a `Result` enum
        }
    }

    /// Shutdown channels between p2p, worker and yourself.
    ///
    /// Can be inlined as it is called only once from very few places.
    #[inline]
    pub async fn shutdown(&mut self) -> Result<()> {
        log::debug!("Sending shutdown command to p2p client.");
        self.p2p.shutdown().await?;

        log::debug!("Closing task output channel.");
        self.task_output_rx.close();

        log::debug!("Closing reqres channel.");
        self.reqres_rx.close();

        Ok(())
    }
}
