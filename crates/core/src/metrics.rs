//! Prometheus metrics for rrelayer.
//!
//! Exposes a process-wide registry plus a `relayer_native_balance` gauge, scraped via the
//! `/metrics` endpoint, so low relayer gas balances can be alerted on in groundcover instead of
//! only surfacing as a `warn!` log in the balance monitor.

use once_cell::sync::Lazy;
use prometheus::{Encoder, GaugeVec, Opts, Registry, TextEncoder};

static REGISTRY: Lazy<Registry> = Lazy::new(Registry::new);

static RELAYER_NATIVE_BALANCE: Lazy<GaugeVec> = Lazy::new(|| {
    let gauge = GaugeVec::new(
        Opts::new(
            "relayer_native_balance",
            "Relayer native (gas) balance in ether, per chain/relayer/address",
        ),
        &["chain_id", "relayer_id", "address"],
    )
    .expect("relayer_native_balance gauge opts are valid");
    REGISTRY
        .register(Box::new(gauge.clone()))
        .expect("relayer_native_balance gauge registers exactly once");
    gauge
});

static RELAYER_NATIVE_MIN_BALANCE: Lazy<GaugeVec> = Lazy::new(|| {
    let gauge = GaugeVec::new(
        Opts::new(
            "relayer_native_min_balance",
            "Minimum recommended relayer native (gas) balance in ether, per chain",
        ),
        &["chain_id"],
    )
    .expect("relayer_native_min_balance gauge opts are valid");
    REGISTRY
        .register(Box::new(gauge.clone()))
        .expect("relayer_native_min_balance gauge registers exactly once");
    gauge
});

/// Record a relayer's current native (gas) balance, in ether.
pub fn record_relayer_native_balance(chain_id: &str, relayer_id: &str, address: &str, ether: f64) {
    RELAYER_NATIVE_BALANCE.with_label_values(&[chain_id, relayer_id, address]).set(ether);
}

/// Record the per-chain minimum recommended native balance, in ether. Lets alerting compare
/// `relayer_native_balance < relayer_native_min_balance` without hard-coding the threshold.
pub fn record_relayer_native_min_balance(chain_id: &str, ether: f64) {
    RELAYER_NATIVE_MIN_BALANCE.with_label_values(&[chain_id]).set(ether);
}

/// Render the registry in Prometheus text exposition format (the `/metrics` body).
pub fn gather() -> String {
    let encoder = TextEncoder::new();
    let mut buffer = Vec::new();
    if let Err(e) = encoder.encode(&REGISTRY.gather(), &mut buffer) {
        tracing::error!("Failed to encode prometheus metrics: {e}");
    }
    String::from_utf8_lossy(&buffer).into_owned()
}
