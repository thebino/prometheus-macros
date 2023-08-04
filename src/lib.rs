//! `prometheus-macros` offers advanced macros for defining [`prometheus`] metrics.
//!
//! This crate extends [`prometheus`] by introducing declarative macros that minimize
//! boilerplate during the declaration and initialization of metrics. Multiple metrics
//! are often needed, as seen for example in contexts like HTTP request
//! where one needs to declare distinct metrics for request count and request latency.
//!
//! Although [`prometheus`] already offers declarative macros for initializing individual
//! metrics, it can still lead to significant boilerplate when declaring multiple metrics.
//!
//! # Example
//!
//! ```
//! use prometheus::{IntGauge, HistogramVec};
//! use prometheus_macros::composite_metric;
//!
//! composite_metric! {
//!     struct CompositeMetric {
//!         #[name = "custom_gauge"]
//!         #[desc = "Example gauge metric"]
//!         custom_gauge: IntGauge,
//!         #[name = "custom_hist_vec"]
//!         #[desc = "Example histogram vec"]
//!         #[labels = ["foo", "bar"]]
//!         #[buckets = [0.01, 0.1, 0.2]]
//!         custom_hist_vec: HistogramVec,
//!     }
//! }
//!
//! let metric = CompositeMetric::register(prometheus::default_registry())
//!     .expect("failed to register metrics to default registry");
//! // access the metrics
//! metric.custom_gauge().set(420);
//! metric.custom_hist_vec().with_label_values(&["a", "b"]).observe(0.5);
//! ```

#![deny(missing_docs)]

use prometheus::{
    self, Counter, CounterVec, Gauge, GaugeVec, Histogram, HistogramOpts, HistogramVec,
    IntCounterVec, IntGauge, IntGaugeVec, Opts as PrometheusOpts,
};

/// Composes multiple prometheus metrics into one struct.
///
/// # Example:
///
/// ```
/// use prometheus::{IntGauge, HistogramVec};
/// use prometheus_macros::composite_metric;
///
/// composite_metric! {
///     struct CompositeMetric {
///         #[name = "custom_gauge"]
///         #[desc = "Example gauge metric"]
///         custom_gauge: IntGauge,
///         #[name = "custom_hist_vec"]
///         #[desc = "Example histogram vec"]
///         #[labels = ["foo", "bar"]]
///         #[buckets = [0.01, 0.1, 0.2]]
///         custom_hist_vec: HistogramVec,
///     }
/// }
///
/// fn collect_metric() {
///     let metric = CompositeMetric::register(prometheus::default_registry())
///         .expect("failed to register metrics to default registry");
///     metric.custom_gauge().set(420);
///     metric.custom_hist_vec().with_label_values(&["a", "b"]).observe(0.5);
/// }
/// ```
#[macro_export]
macro_rules! composite_metric {
    (
        $(#[$m:meta])*
        $v:vis struct $name:ident {
            $(
                #[name = $prom_name:literal]
                #[desc = $prom_desc:literal]
                $(#[labels = $prom_labels:expr])?
                $(#[buckets = $prom_buckets:expr])?
                $metric_name:ident: $metric_ty:ty
            ),+
            $(,)?
        }
    ) => {
        $(#[$m])*
        $v struct $name {
            $(
                $metric_name: $metric_ty,
            )+
        }

        impl $name {
            $v fn register(registry: &::prometheus::Registry) -> ::prometheus::Result<Self> {
                $(
                    let opts = $crate::Opts::new($prom_name, $prom_desc);
                    $(
                        let opts = opts
                            .with_labels(&$prom_labels);
                    )?
                    $(
                        let opts = opts
                            .with_buckets(&$prom_buckets);
                    )?
                    let $metric_name: $metric_ty = opts.try_into().unwrap();
                    registry.register(::std::boxed::Box::new($metric_name.clone()))?;
                )+

                Ok(Self {
                    $(
                        $metric_name
                    ),+
                })
            }


            $(
                $v fn $metric_name (&self) -> &$metric_ty {
                    &self.$metric_name
                }
            )+
        }
    };
}

/// A more generic prometheus options that allow construction of both scalar and vector metrics.
#[derive(Default)]
pub struct Opts<'a> {
    name: &'a str,
    desc: &'a str,
    labels: Option<&'a [&'a str]>,
    buckets: Option<&'a [f64]>,
}

impl<'a> Opts<'a> {
    /// Create a new generic metric option based name, helper text and optional labels.
    pub fn new(name: &'a str, desc: &'a str) -> Self {
        Self {
            name,
            desc,
            ..Self::default()
        }
    }

    /// Attaches labels to the options.
    pub fn with_labels(mut self, labels: &'a [&'a str]) -> Self {
        self.labels = labels.into();
        self
    }

    /// Attaches buckets to the options.
    pub fn with_buckets(mut self, buckets: &'a [f64]) -> Self {
        self.buckets = buckets.into();
        self
    }
}

macro_rules! impl_try_from {
    ($ident:ident, $opts:ident $(,)? $($param:ident),*) => {
        impl TryFrom<Opts<'_>> for $ident {
            type Error = prometheus::Error;
            fn try_from(opts: Opts<'_>) -> Result<Self, Self::Error> {
                #[allow(unused_mut)]
                let mut prom_opts = <$opts>::new(opts.name, opts.desc);
                $(
                    if let Some(param) = opts.$param {
                        prom_opts.$param = param.into();
                    }
                )*
                <$ident>::with_opts(prom_opts.into())
            }
        }
    };
}

impl_try_from!(Counter, PrometheusOpts);
impl_try_from!(IntGauge, PrometheusOpts);
impl_try_from!(Gauge, PrometheusOpts);
impl_try_from!(Histogram, HistogramOpts, buckets);

macro_rules! impl_try_from_vec {
    ($ident:ident, $opts:ident $(,)? $($param:ident),*) => {
        impl TryFrom<Opts<'_>> for $ident {
            type Error = prometheus::Error;
            fn try_from(opts: Opts<'_>) -> Result<Self, Self::Error> {
                #[allow(unused_mut)]
                let mut prom_opts = <$opts>::new(opts.name, opts.desc);
                $(
                    if let Some(param) = opts.$param {
                        prom_opts.$param = param.into();
                    }
                )*
                <$ident>::new(
                    prom_opts.into(),
                    opts.labels.ok_or_else(|| {
                        prometheus::Error::Msg("vec requires one or more labels".to_owned())
                    })?,
                )
            }
        }
    };
}

impl_try_from_vec!(IntCounterVec, PrometheusOpts);
impl_try_from_vec!(CounterVec, PrometheusOpts);
impl_try_from_vec!(GaugeVec, PrometheusOpts);
impl_try_from_vec!(IntGaugeVec, PrometheusOpts);
impl_try_from_vec!(HistogramVec, HistogramOpts, buckets);

#[cfg(test)]
mod tests {
    use crate::*;
    use prometheus::*;

    fn parse_name(enc: &str) -> &str {
        enc.lines()
            .next()
            .expect("mutliple lines")
            .split(' ')
            .nth(2)
            .expect("description line")
    }

    fn parse_description(enc: &str) -> &str {
        enc.lines()
            .next()
            .expect("mutliple lines")
            .split(' ')
            .nth(3)
            .expect("description line")
    }

    fn parse_type(enc: &str) -> &str {
        enc.lines()
            .nth(1)
            .expect("mutliple lines")
            .split(' ')
            .nth(3)
            .expect("type line")
    }

    fn parse_labels(enc: &str) -> Vec<&str> {
        let (_, s) = enc
            .lines()
            .nth(2)
            .expect("mutliple lines")
            .split_once('{')
            .unwrap();
        let (s, _) = s.split_once('}').unwrap();
        s.split(',')
            .filter_map(|s| {
                let (l, _) = s.split_once('=')?;
                Some(l)
            })
            .collect()
    }

    fn parse_buckets(enc: &str) -> Vec<&str> {
        enc.lines()
            .skip(2)
            .filter_map(|s| {
                let (_, s) = s.split_once("le=")?;
                let s = s.split('\"').nth(1)?;
                Some(s)
            })
            .collect()
    }

    #[test]
    fn compose_metric_and_encode() {
        composite_metric! {
            struct CompositeMetric {
                #[name = "example_gauge_1"]
                #[desc = "description"]
                gauge_metric_1: Gauge,
                #[name = "example_gauge_2"]
                #[desc = "description"]
                gauge_metric_2: Gauge,
            }
        }

        let reg = Registry::new();
        let metric = CompositeMetric::register(&reg).unwrap();
        metric.gauge_metric_1().inc();
        metric.gauge_metric_2().inc();

        let enc = TextEncoder::new().encode_to_string(&reg.gather()).unwrap();

        assert_eq!(
            enc,
            r#"# HELP example_gauge_1 description
# TYPE example_gauge_1 gauge
example_gauge_1 1
# HELP example_gauge_2 description
# TYPE example_gauge_2 gauge
example_gauge_2 1
"#
        );
    }

    #[test]
    fn with_name_desc() {
        composite_metric! {
            struct CompositeMetric {
                #[name = "example_gauge"]
                #[desc = "description"]
                gauge_metric: Gauge,
            }
        }
        let reg = Registry::new();
        let metric = CompositeMetric::register(&reg).unwrap();
        metric.gauge_metric().inc();
        let enc = TextEncoder::new().encode_to_string(&reg.gather()).unwrap();

        assert_eq!(parse_name(&enc), "example_gauge");
        assert_eq!(parse_description(&enc), "description");
        assert_eq!(parse_type(&enc), "gauge");
    }

    #[test]
    fn with_labels() {
        composite_metric! {
            struct CompositeMetric {
                #[name = "example_gauge_vec"]
                #[desc = "description"]
                #[labels = ["label1", "label2"]]
                gauge_vec_metric: GaugeVec,
            }
        }
        let reg = Registry::new();
        let metric = CompositeMetric::register(&reg).unwrap();
        metric
            .gauge_vec_metric()
            .with_label_values(&["a", "b"])
            .inc();
        let enc = TextEncoder::new().encode_to_string(&reg.gather()).unwrap();

        assert_eq!(parse_name(&enc), "example_gauge_vec");
        assert_eq!(parse_description(&enc), "description");
        assert_eq!(parse_type(&enc), "gauge");
        assert_eq!(parse_labels(&enc), vec!["label1", "label2"]);
    }

    #[test]
    fn with_buckets() {
        composite_metric! {
            struct CompositeMetric {
                #[name = "example_hist"]
                #[desc = "description"]
                #[buckets = [0.1, 0.5]]
                hist_metric: Histogram,
            }
        }
        let reg = Registry::new();
        let metric = CompositeMetric::register(&reg).unwrap();
        metric.hist_metric().observe(0.1);
        let enc = TextEncoder::new().encode_to_string(&reg.gather()).unwrap();

        assert_eq!(parse_name(&enc), "example_hist");
        assert_eq!(parse_description(&enc), "description");
        assert_eq!(parse_type(&enc), "histogram");
        assert_eq!(parse_buckets(&enc), vec!["0.1", "0.5", "+Inf"]);
    }
}
