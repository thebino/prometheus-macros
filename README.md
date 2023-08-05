# Prometheus Macros 

<a href="https://github.com/heat1q/prometheus-macros/actions/workflows/rust.yml">
<img src="https://github.com/heat1q/prometheus-macros/actions/workflows/rust.yml/badge.svg" />
</a>
<a href="https://crates.io/crates/prometheus-macros">
<img src="https://img.shields.io/crates/v/prometheus-macros.svg" />
</a>
<a href="https://docs.rs/prometheus-macros">
<img src="https://docs.rs/prometheus-macros/badge.svg" />
</a>
<br/>

## Motivation

This crate extends [`prometheus`](https://docs.rs/prometheus/latest/prometheus/) by introducing declarative macros that minimize
boilerplate during the declaration and initialization of metrics. Multiple metrics
are often needed, as seen for example in contexts like HTTP request
where one needs to declare distinct metrics for request count and request latency.

Although [`prometheus`](https://docs.rs/prometheus/latest/prometheus/) already offers 
declarative macros for initializing individual metrics, it can still lead to significant 
boilerplate when declaring multiple metrics.

## Example 
```rust
use prometheus::{IntGauge, HistogramVec};
use prometheus_macros::composite_metric;

composite_metric! {
    struct CompositeMetric {
        #[name = "custom_gauge"]
        #[desc = "Example gauge metric"]
        custom_gauge: IntGauge,
        #[name = "custom_hist_vec"]
        #[desc = "Example histogram vec"]
        #[labels = ["foo", "bar"]]
        #[buckets = [0.01, 0.1, 0.2]]
        custom_hist_vec: HistogramVec,
    }
}

fn main() {
    let metric = CompositeMetric::register(prometheus::default_registry())
        .expect("failed to register metrics to default registry");
    // access the metrics
    metric.custom_gauge().set(420);
    metric.custom_hist_vec().with_label_values(&["a", "b"]).observe(0.5);
}
```
