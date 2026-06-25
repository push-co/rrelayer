use std::{sync::Arc, time::Duration};

use tracing::{error, info};

use crate::{postgres::PostgresError, provider::EvmProvider, PostgresClient, WalletError};

#[derive(thiserror::Error, Debug)]
pub enum CanaryError {
    #[error("KMS signing canary failed: {0}")]
    Sign(#[from] WalletError),
    #[error("canary could not load relayers: {0}")]
    RelayerLookup(#[from] PostgresError),
}

pub async fn run_kms_permission_canary(
    providers: &[EvmProvider],
    db: &PostgresClient,
) -> Result<(), CanaryError> {
    for provider in providers {
        let relayers = db.get_all_relayers_for_chain(&provider.chain_id).await?;
        for relayer in relayers {
            if let Err(e) = provider.sign_text(&relayer, "rrelayer-canary").await {
                error!(
                    relayer  = %relayer.address,
                    chain_id = %relayer.chain_id,
                    error    = %e,
                    "KMS signing canary FAILED — principal lacks kms:Sign or key unusable"
                );
                return Err(CanaryError::Sign(e));
            }
        }
    }
    Ok(())
}

pub async fn run_kms_permission_canary_task(
    providers: Arc<Vec<EvmProvider>>,
    db: Arc<PostgresClient>,
) {
    info!("Starting KMS permission canary background task");
    tokio::spawn(async move {
        let interval = Duration::from_secs(36000);
        loop {
            if let Err(e) = run_kms_permission_canary(&providers, &db).await {
                error!(error = %e, "KMS permission canary cycle failed");
            }
            tokio::time::sleep(interval).await;
        }
    });
    info!("Started KMS permission canary background task");
}
