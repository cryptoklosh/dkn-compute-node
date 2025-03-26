use colored::Colorize;
use std::time::Duration;

use crate::{
    utils::{get_points, DriaRPC},
    DriaComputeNode, DRIA_COMPUTE_NODE_VERSION,
};

/// Number of seconds such that if the last heartbeat ACK is older than this, the node is considered unreachable.
const HEARTBEAT_LIVENESS_SECS: Duration = Duration::from_secs(150);

impl DriaComputeNode {
    /// Returns the task count within the channels, `single` and `batch`.
    #[inline(always)]
    pub fn get_pending_task_count(&self) -> [usize; 2] {
        [
            self.pending_tasks_single.len(),
            self.pending_tasks_batch.len(),
        ]
    }

    /// Peer refresh simply reports the peer count to the user.
    pub(crate) async fn handle_diagnostic_refresh(&self) {
        let mut diagnostics = vec![format!("Diagnostics (v{}):", DRIA_COMPUTE_NODE_VERSION)];

        // print steps
        if let Ok(steps) = get_points(&self.config.address).await {
            let earned = steps.score - self.initial_steps;
            diagnostics.push(format!(
                "$DRIA Points: {} total, {} earned in this run, within top {}%",
                steps.score, earned, steps.percentile
            ));
        }

        // completed tasks count is printed as well in debug
        if log::log_enabled!(log::Level::Debug) {
            diagnostics.push(format!(
                "Completed Tasks (single/batch): {} / {}",
                self.completed_tasks_single, self.completed_tasks_batch
            ));
        }

        // print peer id and address
        diagnostics.push(format!("Peer ID: {}", self.config.peer_id));
        diagnostics.push(format!("Address: 0x{}", self.config.address));

        // print models
        diagnostics.push(format!(
            "Models: {}",
            self.config
                .workflows
                .models
                .iter()
                .map(|(p, m)| format!("{}/{}", p, m))
                .collect::<Vec<String>>()
                .join(", ")
        ));

        // if we have not received pings for a while, we are considered offline
        let is_offline = chrono::Utc::now() > self.last_heartbeat_at + HEARTBEAT_LIVENESS_SECS;

        // if we have not yet received a heartbeat response, we are still connecting
        if self.num_heartbeats == 0 {
            // if we didnt have any pings, we might still be connecting
            diagnostics.push(format!("Node Status: {}", "CONNECTING".yellow()));
        } else {
            diagnostics.push(format!(
                "Node Status: {}",
                if is_offline {
                    "OFFLINE".red()
                } else {
                    "ONLINE".green()
                }
            ));
        }

        log::info!("{}", diagnostics.join("\n  "));

        // if offline, print this error message as well
        if is_offline {
            log::error!(
                "Node has not received any pings for at least {} seconds & it may be unreachable!\nPlease restart your node!",
                HEARTBEAT_LIVENESS_SECS.as_secs()
            );
        }
    }

    /// Dials the existing RPC node if we are not connected to it.
    ///
    /// If there is an error while doing that,
    /// it will try to get a new RPC node and dial it.
    pub(crate) async fn handle_available_nodes_refresh(&mut self) {
        log::debug!("Checking RPC connections for diagnostics.");

        // check if we are connected
        let is_connected = self
            .p2p
            .is_connected(self.dria_rpc.peer_id)
            .await
            .unwrap_or(false);

        // if we are not connected, try to dial it again
        if !is_connected {
            log::info!("Dialling RPC at: {}", self.dria_rpc.addr);
            if let Err(err) = self
                .dial_with_timeout(self.dria_rpc.peer_id, self.dria_rpc.addr.clone())
                .await
            {
                // if we also cannot dial it, get a new RPC node
                log::warn!(
                    "Could not dial to RPC at: {}: {err:?}\nWill get a new RPC node.",
                    self.dria_rpc.addr,
                );
                if let Ok(new_rpc) = DriaRPC::new(self.dria_rpc.network).await {
                    self.dria_rpc = new_rpc;

                    // now dial this new RPC again
                    if let Err(err) = self
                        .dial_with_timeout(self.dria_rpc.peer_id, self.dria_rpc.addr.clone())
                        .await
                    {
                        // worst-case we cant dial this one too, just leave it for the next diagnostic
                        log::error!("Could not dial the new RPC: {err:?}\nWill try again in the next diagnostic refresh.");
                    }
                } else {
                    log::error!("Could not get a new RPC node!\nWill try again in the next diagnostic refresh.");
                }
            } else {
                log::info!("Successfully dialled to RPC at: {}", self.dria_rpc.addr);
            }
        } else {
            log::debug!("Connection with {} is intact.", self.dria_rpc.peer_id);
        }
    }
}
