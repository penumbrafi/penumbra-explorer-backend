use async_graphql::Context;
use sqlx::{Pool, Postgres};
use tokio::sync::broadcast;
use tracing::{debug, info, warn};

pub mod ibc;
mod listen;
mod triggers;
pub mod validator;
use ibc::IbcTransactionEvent;
pub use triggers::start;
use validator::{ChainParametersEvent, ValidatorBlockEvent};

#[derive(Clone)]
pub struct PubSub {
    blocks_tx: broadcast::Sender<i64>,
    transactions_tx: broadcast::Sender<i64>,
    transaction_count_tx: broadcast::Sender<i64>,
    ibc_transactions_tx: broadcast::Sender<IbcTransactionEvent>,
    total_shielded_volume_tx: broadcast::Sender<String>,
    /// Single global broadcast for all validator block events.
    /// Subscribers filter by validator_id at the resolver layer.
    /// (Was previously a HashMap per-validator_id which never freed entries
    /// and leaked PgListener connections + tasks; ~675 events/s × N visited
    /// validators of state forever.)
    validator_blocks_tx: broadcast::Sender<ValidatorBlockEvent>,
    chain_parameters_tx: broadcast::Sender<ChainParametersEvent>,
}

impl Default for PubSub {
    fn default() -> Self {
        Self::new()
    }
}

impl PubSub {
    #[must_use]
    pub fn new() -> Self {
        let (blocks_tx, _) = broadcast::channel(1000);
        let (transactions_tx, _) = broadcast::channel(1000);
        let (transaction_count_tx, _) = broadcast::channel(1000);
        let (ibc_transactions_tx, _) = broadcast::channel(1000);
        let (total_shielded_volume_tx, _) = broadcast::channel(1000);
        let (validator_blocks_tx, _) = broadcast::channel(2048);
        let (chain_parameters_tx, _) = broadcast::channel(1000);

        Self {
            blocks_tx,
            transactions_tx,
            transaction_count_tx,
            ibc_transactions_tx,
            total_shielded_volume_tx,
            validator_blocks_tx,
            chain_parameters_tx,
        }
    }

    #[must_use]
    pub fn blocks_subscribe(&self) -> broadcast::Receiver<i64> {
        self.blocks_tx.subscribe()
    }

    #[must_use]
    pub fn transactions_subscribe(&self) -> broadcast::Receiver<i64> {
        self.transactions_tx.subscribe()
    }

    #[must_use]
    pub fn transaction_count_subscribe(&self) -> broadcast::Receiver<i64> {
        self.transaction_count_tx.subscribe()
    }

    #[must_use]
    pub fn ibc_transactions_subscribe(&self) -> broadcast::Receiver<IbcTransactionEvent> {
        self.ibc_transactions_tx.subscribe()
    }

    #[must_use]
    pub fn total_shielded_volume_subscribe(&self) -> broadcast::Receiver<String> {
        self.total_shielded_volume_tx.subscribe()
    }

    #[must_use]
    pub fn chain_parameters_subscribe(&self) -> broadcast::Receiver<ChainParametersEvent> {
        self.chain_parameters_tx.subscribe()
    }

    /// Subscribe to ALL validator block events. Caller filters by
    /// validator_id at the resolver layer. The single global PgListener is
    /// started once via `start_subscriptions`, not per subscriber.
    #[must_use]
    pub fn validator_blocks_subscribe(&self) -> broadcast::Receiver<ValidatorBlockEvent> {
        self.validator_blocks_tx.subscribe()
    }

    pub fn publish_block(&self, height: i64) {
        match self.blocks_tx.send(height) {
            Ok(_) => debug!("Published block update: {}", height),
            Err(e) => {
                let receiver_count = self.blocks_tx.receiver_count();
                if receiver_count == 0 {
                    debug!("No receivers for block update");
                } else {
                    warn!("Failed to publish block update: {}", e);
                }
            }
        }
    }

    pub fn publish_transaction(&self, id: i64) {
        match self.transactions_tx.send(id) {
            Ok(_) => debug!("Published transaction update: {}", id),
            Err(e) => {
                let receiver_count = self.transactions_tx.receiver_count();
                if receiver_count == 0 {
                    debug!("No receivers for transaction update");
                } else {
                    warn!("Failed to publish transaction update: {}", e);
                }
            }
        }
    }

    pub fn publish_transaction_count(&self, count: i64) {
        match self.transaction_count_tx.send(count) {
            Ok(_) => debug!("Published transaction count update: {}", count),
            Err(e) => {
                let receiver_count = self.transaction_count_tx.receiver_count();
                if receiver_count == 0 {
                    debug!("No receivers for transaction count update");
                } else {
                    warn!("Failed to publish transaction count update: {}", e);
                }
            }
        }
    }

    pub fn publish_ibc_transaction(&self, event: IbcTransactionEvent) {
        match self.ibc_transactions_tx.send(event) {
            Ok(_) => debug!("Published IBC transaction update"),
            Err(e) => {
                let receiver_count = self.ibc_transactions_tx.receiver_count();
                if receiver_count == 0 {
                    debug!("No receivers for IBC transaction update");
                } else {
                    warn!("Failed to publish IBC transaction update: {}", e);
                }
            }
        }
    }

    pub fn publish_total_shielded_volume(&self, value: String) {
        let value_clone = value.clone();
        match self.total_shielded_volume_tx.send(value) {
            Ok(_) => debug!("Published total shielded volume update: {}", value_clone),
            Err(e) => {
                let receiver_count = self.total_shielded_volume_tx.receiver_count();
                if receiver_count == 0 {
                    debug!("No receivers for total shielded volume update");
                } else {
                    warn!("Failed to publish total shielded volume update: {}", e);
                }
            }
        }
    }

    pub fn publish_validator_block(&self, event: ValidatorBlockEvent) {
        match self.validator_blocks_tx.send(event.clone()) {
            Ok(_) => debug!(
                "Published validator block update for {}: height {} signed {}",
                event.validator_id, event.block_height, event.signed
            ),
            Err(e) => {
                let receiver_count = self.validator_blocks_tx.receiver_count();
                if receiver_count == 0 {
                    debug!("No receivers for validator block update");
                } else {
                    warn!("Failed to publish validator block update: {}", e);
                }
            }
        }
    }

    pub fn publish_chain_parameters(&self, event: &ChainParametersEvent) {
        match self.chain_parameters_tx.send(event.clone()) {
            Ok(_) => debug!(
                "Published chain parameters update - block height: {}, epoch: {}",
                event.current_block_height, event.current_epoch
            ),
            Err(e) => {
                let receiver_count = self.chain_parameters_tx.receiver_count();
                if receiver_count == 0 {
                    debug!("No receivers for chain parameters update");
                } else {
                    warn!("Failed to publish chain parameters update: {}", e);
                }
            }
        }
    }

    #[must_use]
    pub fn from_context<'a>(ctx: &'a Context<'_>) -> Option<&'a Self> {
        ctx.data_opt::<Self>()
    }

    /// Setup database triggers for real-time notifications
    ///
    /// # Errors
    /// Returns an error only in case of critical database connection issues.
    /// Missing tables result in graceful fallback to polling mode.
    pub async fn setup_triggers(&self, pool: &Pool<Postgres>) -> Result<(), anyhow::Error> {
        info!("Setting up database triggers with retry logic");

        match triggers::setup_notification_triggers_with_retry(pool).await {
            Ok(()) => {
                info!("Database triggers setup completed successfully");
                Ok(())
            }
            Err(e) => {
                warn!(
                    "Failed to set up database triggers: {}. Continuing with polling-only mode.",
                    e
                );
                // Don't return error - this allows the application to continue with polling
                // This is especially important for fresh deployments where tables don't exist yet
                Ok(())
            }
        }
    }

    pub fn start_subscriptions(&self, pool: &Pool<Postgres>) {
        info!("Starting subscription listeners");

        let pubsub_clone = self.clone();
        let pool_clone = pool.clone();

        tokio::spawn(async move {
            start(pubsub_clone, pool_clone).await;
        });

        let pubsub_clone = self.clone();
        let pool_clone = pool.clone();
        tokio::spawn(async move {
            validator::listen_chain_parameters(pubsub_clone, pool_clone).await;
        });

        // Single global validator-block listener — replaces the per-id
        // listener-per-subscription pattern that leaked PgListener
        // connections + tasks for every distinct validator_id ever queried.
        let pubsub_clone = self.clone();
        let pool_clone = pool.clone();
        tokio::spawn(async move {
            validator::listen_validator_blocks(pubsub_clone, pool_clone).await;
        });
    }
}
