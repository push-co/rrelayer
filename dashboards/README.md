# Observability — rrelayer dashboards & alerts

Grafana dashboard and Prometheus alert rules for rrelayer, built from the metrics it exposes on
`/metrics` (mounted on the API server, same port as the rrelayer API — `8000` in the push deploy).
Mirrors the per-service dashboards in `push-backend` (`crates/*/dashboards`).

## Files

| File | What |
|---|---|
| `relayer.json` | Grafana dashboard — relayer native (gas) balances vs the per-chain minimum, plus a freshness/below-minimum overview. |
| `relayer-alerts.yml` | Prometheus-format alert rules (`RelayerNativeBalanceLow`, `RelayerBalanceMetricStale`). Portable to Prometheus / Grafana ruler / groundcover. |

## Metrics

Emitted by the `balance_monitor` background task (~every 10 min):

- `relayer_native_balance{chain_id, relayer_id, address}` — relayer native (gas) balance in ether.
- `relayer_native_min_balance{chain_id}` — per-chain minimum recommended balance, so alerting compares
  `relayer_native_balance < relayer_native_min_balance` without a hard-coded floor.

## Import

Grafana → Dashboards → Import → upload `relayer.json` → pick your Prometheus datasource. The dashboard
uses `$env` / `$namespace` template variables (same convention as the push-backend dashboards); they're
derived from the `relayer_native_balance` series itself, so it doesn't depend on the pod's `app` label.

> **Requires** Prometheus/groundcover to scrape rrelayer's `/metrics`, and the scraped series to carry
> `env` / `namespace` labels (added by the cluster scrape, same as the other push services). The rrelayer
> chart does **not** currently have `prometheus.io/scrape` annotations — see the k8s notes below.
