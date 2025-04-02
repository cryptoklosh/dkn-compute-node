use libp2p::StreamProtocol;
use std::env;

#[derive(Clone, Debug)]
pub struct DriaP2PProtocol {
    /// Main protocol name, e.g. `dria`.
    pub name: String,
    /// Version of the protocol, e.g. `0.2`.
    /// By default, this is set to the current `major.minor` version of the crate.
    pub version: String,
    /// Identity protocol string to be used for the Identity behaviour.
    ///
    /// This is usually `{name}/{version}`.
    pub identity: String,
    /// Request-response protocol, must match with other peers in the network.
    ///
    /// This is usually `/{name}/rr/{version}`, notice the `/` at the start
    /// which is mandatory for a `StreamProtocol`.
    ///
    pub request_response: StreamProtocol,
}

impl std::fmt::Display for DriaP2PProtocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.identity)
    }
}

impl Default for DriaP2PProtocol {
    /// Creates a new instance of the protocol with the default name `dria`.
    fn default() -> Self {
        Self::new_major_minor("dria")
    }
}

impl DriaP2PProtocol {
    /// Creates a new instance of the protocol with the given `name` and `version`.
    pub fn new(name: impl ToString, version: impl ToString) -> Self {
        let name = name.to_string();
        let version = version.to_string();

        let identity = format!("{}/{}", name, version);
        let request_response =
            StreamProtocol::try_from_owned(format!("/{}/rr/{}", name, version)).unwrap();

        Self {
            name,
            version,
            identity,
            request_response,
        }
    }

    /// Creates a new instance of the protocol with the given `name` and the current version as per Cargo.toml.
    /// The verison is represented with `major.minor` version numbers.
    pub fn new_major_minor(name: &str) -> Self {
        const VERSION: &str = concat!(
            env!("CARGO_PKG_VERSION_MAJOR"),
            ".",
            env!("CARGO_PKG_VERSION_MINOR")
        );

        Self::new(name, VERSION)
    }

    /// Returns the identity protocol, e.g. `dria/0.2`.
    pub fn identity(&self) -> String {
        self.identity.clone()
    }

    /// Returns the request-response protocol, e.g. `/dria/rr/0.2`.
    pub fn request_response(&self) -> StreamProtocol {
        self.request_response.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let protocol = DriaP2PProtocol::new("test", "1.0");
        assert_eq!(protocol.name, "test");
        assert_eq!(protocol.version, "1.0");
        assert_eq!(protocol.identity, "test/1.0");
        assert_eq!(protocol.request_response.to_string(), "/test/rr/1.0");
    }

    #[test]
    fn test_new_major_minor() {
        let protocol = DriaP2PProtocol::new_major_minor("test");
        assert_eq!(protocol.name, "test");
        assert_eq!(
            protocol.version,
            concat!(
                env!("CARGO_PKG_VERSION_MAJOR"),
                ".",
                env!("CARGO_PKG_VERSION_MINOR")
            )
        );
        assert_eq!(protocol.identity, format!("test/{}", protocol.version));
    }
}
