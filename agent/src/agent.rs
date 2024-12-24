use crate::batch_maker::{Batch, Transaction};
use async_trait::async_trait;
use bytes::Bytes;
use config::{Committee, Parameters, AgentId};
use crypto::{PublicKey};
use futures::sink::SinkExt as _;
use log::{error, info, warn};
use network::{MessageHandler, Receiver, Writer};
use std::error::Error;
use tokio::sync::mpsc::{channel, Sender};

/// The default channel capacity for each channel of the agent.
pub const CHANNEL_CAPACITY: usize = 1_000;

/// Indicates a serialized `WorkerPrimaryMessage` message.
pub type SerializedBatchDigestMessage = Vec<u8>;

pub struct Agent {
    /// The public key of this authority.
    name: PublicKey,
    /// The id of this agent.
    id: AgentId,
    /// The committee information.
    committee: Committee,
    /// The configuration parameters.
    parameters: Parameters,
}

impl Agent {
    pub fn spawn(
        name: PublicKey,
        id: AgentId,
        committee: Committee,
        parameters: Parameters,
    ) {
        // Define a agent instance.
        let agent = Self {
            name,
            id,
            committee,
            parameters,
            // store,
        };

        // Spawn all agent tasks.
        let (tx_primary, rx_primary) = channel(CHANNEL_CAPACITY);
        agent.handle_clients_transactions(tx_primary.clone());

        // NOTE: This log entry is used to compute performance.
        info!(
            "Agent {} successfully booted on {}",
            id,
            agent
                .committee
                .agent(&agent.name, &agent.id)
                .expect("Our public key or agent id is not in the committee")
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
            .agent(&self.name, &self.id)
            .expect("Our public key or agent id is not in the committee")
            .transactions;
        address.set_ip("0.0.0.0".parse().unwrap());
        Receiver::spawn(
            address,
            /* handler */ TxReceiverHandler { tx_batch_maker },
        );

        info!(
            "Agent {} listening to client transactions on {}",
            self.id, address
        );
    }
}

/// Defines how the network receiver handles incoming transactions.
#[derive(Clone)]
struct TxReceiverHandler {
    tx_batch_maker: Sender<Transaction>,
}

#[async_trait]
impl MessageHandler for TxReceiverHandler {
    async fn dispatch(&self, _writer: &mut Writer, message: Bytes) -> Result<(), Box<dyn Error>> {
        // Send the transaction to the batch maker.
        self.tx_batch_maker
            .send(message.to_vec())
            .await
            .expect("Failed to send transaction");

        // Give the change to schedule other tasks.
        tokio::task::yield_now().await;
        Ok(())
    }
}