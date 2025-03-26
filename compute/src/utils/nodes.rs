use dkn_p2p::{
    libp2p::{Multiaddr, PeerId},
    DriaNetworkType, DriaNodes,
};
use dkn_utils::parse_vec;
use eyre::Result;

/// Refresh available nodes using the API.
pub async fn refresh_dria_nodes(nodes: &mut DriaNodes) -> Result<()> {
    #[derive(serde::Deserialize, Debug)]
    struct DriaNodesApiResponse {
        pub rpcs: Vec<String>,
        #[serde(rename = "rpcAddrs")]
        pub rpc_addrs: Vec<Multiaddr>,
    }

    // url to be used is determined by the network type
    let url = match nodes.network {
        DriaNetworkType::Community => "https://dkn.dria.co/available-nodes",
        DriaNetworkType::Pro => "https://dkn.dria.co/sdk/available-nodes",
        DriaNetworkType::Test => "https://dkn.dria.co/test/available-nodes",
    };

    // make the request
    let response = reqwest::get(url).await?;
    let response_body = response.json::<DriaNodesApiResponse>().await?;

    nodes.rpc_addrs.extend(response_body.rpc_addrs);
    nodes
        .rpc_peerids
        .extend(parse_vec::<PeerId>(response_body.rpcs).unwrap_or_else(|e| {
            log::error!("Failed to parse rpc peerids: {}", e);
            vec![]
        }));

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_refresh_dria_nodes() {
        let mut nodes = DriaNodes::new(DriaNetworkType::Community);
        refresh_dria_nodes(&mut nodes).await.unwrap();
        assert!(!nodes.rpc_addrs.is_empty());
        assert!(!nodes.rpc_peerids.is_empty());

        let mut nodes = DriaNodes::new(DriaNetworkType::Pro);
        refresh_dria_nodes(&mut nodes).await.unwrap();
        assert!(!nodes.rpc_addrs.is_empty());
        assert!(!nodes.rpc_peerids.is_empty());
    }
}
