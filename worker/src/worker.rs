use async_trait::async_trait;
use bytes::Bytes;
use config::{Committee, Parameters, WorkerId};
use futures::sink::SinkExt as _;
use log::{error, info, warn};
use network::{MessageHandler, Receiver, Writer};
use std::error::Error;
use tokio::sync::mpsc::{channel, Sender};

/// The default channel capacity for each channel of the worker.
pub const CHANNEL_CAPACITY: usize = 1_000;

/// Indicates a serialized `WorkerPrimaryMessage` message.
pub type SerializedBatchDigestMessage = Vec<u8>;

pub struct Worker {
    /// The public key of this authority.
    name: PublicKey,
    /// The id of this worker.
    id: WorkerId,
    /// The committee information.
    committee: Committee,
    /// The configuration parameters.
    parameters: Parameters,
}

impl Worker {
    pub fn spawn(
        name: PublicKey,
        id: WorkerId,
        committee: Committee,
        parameters: Parameters,
    ) {
        // Define a worker instance.
        let worker = Self {
            name,
            id,
            committee,
            parameters,
            // store,
        };

        // Spawn all worker tasks.
        let (tx_primary, rx_primary) = channel(CHANNEL_CAPACITY);
        worker.handle_clients_transactions(tx_primary.clone());

        // NOTE: This log entry is used to compute performance.
        info!(
            "Worker {} successfully booted on {}",
            id,
            worker
                .committee
                .worker(&worker.name, &worker.id)
                .expect("Our public key or worker id is not in the committee")
                .transactions
                .ip()
        );
    }

    /// Spawn all tasks responsible to handle clients transactions.
    fn handle_clients_transactions(&self, tx_primary: Sender<SerializedBatchDigestMessage>) {
        let (tx_batch_maker, rx_batch_maker) = channel(CHANNEL_CAPACITY);

        // We first receive clients' transactions from the network.
        let mut address = self
            .committee
            .worker(&self.name, &self.id)
            .expect("Our public key or worker id is not in the committee")
            .transactions;
        address.set_ip("0.0.0.0".parse().unwrap());
        Receiver::spawn(
            address,
            /* handler */ TxReceiverHandler { tx_batch_maker },
        );

        info!(
            "Worker {} listening to client transactions on {}",
            self.id, address
        );
    }
}