use crypto::PublicKey;
use log::info;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::fs::{self, OpenOptions};
use std::io::BufWriter;
use std::io::Write as _;
use std::net::SocketAddr;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Node {0} is not in the committee")]
    NotInCommittee(PublicKey),

    #[error("Unknown worker id {0}")]
    UnknownWorker(WorkerId),

    #[error("Failed to read config file '{file}': {message}")]
    ImportError { file: String, message: String },

    #[error("Failed to write config file '{file}': {message}")]
    ExportError { file: String, message: String },
}

pub type Stake = u32;
pub type WorkerId = u32;

#[derive(Clone, Deserialize)]
pub struct PrimaryAddresses {
    /// Address to receive messages from other primaries (WAN).
    pub primary_to_primary: SocketAddr,
    /// Address to receive messages from our workers (LAN).
    pub worker_to_primary: SocketAddr,
}

#[derive(Clone, Deserialize, Eq, Hash, PartialEq)]
pub struct WorkerAddresses {
    /// Address to receive client transactions (WAN).
    pub transactions: SocketAddr,
    /// Address to receive messages from other workers (WAN).
    pub worker_to_worker: SocketAddr,
    /// Address to receive messages from our primary (LAN).
    pub primary_to_worker: SocketAddr,
}

#[derive(Clone, Deserialize)]
pub struct Authority {
    /// The voting power of this authority.
    pub stake: Stake,
    /// The network addresses of the primary.
    pub primary: PrimaryAddresses,
    /// Map of workers' id and their network addresses.
    pub workers: HashMap<WorkerId, WorkerAddresses>,
}

#[derive(Clone, Deserialize)]
pub struct Committee {
    pub authorities: BTreeMap<PublicKey, Authority>,
}

pub type PublicKey = [u8; 32];

impl Committee {
    /// Returns the addresses of a specific worker (`id`) of a specific authority (`to`).
    pub fn worker(&self, to: &PublicKey, id: &WorkerId) -> Result<WorkerAddresses, ConfigError> {
        self.authorities
            .iter()
            .find(|(name, _)| name == &to)
            .map(|(_, authority)| authority)
            .ok_or_else(|| ConfigError::NotInCommittee(*to))?
            .workers
            .iter()
            .find(|(worker_id, _)| worker_id == &id)
            .map(|(_, worker)| worker.clone())
            .ok_or_else(|| ConfigError::NotInCommittee(*to))
    }
}