use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Topic used within [`crate::DriaMessage`] for specs messages.
pub const SPECS_TOPIC: &str = "specs";

#[derive(Serialize, Deserialize)]
pub struct SpecsRequest {
    /// UUID of the specs request, prevents replays.
    pub specs_id: Uuid,
    /// Node specs.
    pub specs: Specs,
    /// Address of the node, used by frontend etc.
    /// instead of using the peer id.
    pub address: String,
}

#[derive(Serialize, Deserialize)]
pub struct SpecsResponse {
    /// UUID of the specs request, prevents replays.
    pub specs_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Specs {
    /// Total memory in bytes
    pub total_mem: u64,
    /// Free memory in bytes
    pub free_mem: u64,
    /// Number of physical CPU cores.
    pub num_cpus: Option<usize>,
    /// Global CPU usage, in percentage.
    pub cpu_usage: f32,
    /// Operating system name, e.g. `linux`, `macos`, `windows`.
    pub os: String,
    /// CPU architecture, e.g. `x86_64`, `aarch64`.
    pub arch: String,
    /// Public IP lookup response.
    pub lookup: Option<public_ip_address::response::LookupResponse>,
    /// Models server by this node.
    pub models: Vec<String>,
    /// Node version, e.g. `0.1.0`.
    pub version: String,
    // GPU adapter infos, showing information about the available GPUs.
    // gpus: Vec<wgpu::AdapterInfo>,
}
