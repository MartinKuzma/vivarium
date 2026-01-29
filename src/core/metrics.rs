use std::collections::HashMap;
use crate::core::snapshot::MetricsSnapshot;
use rmcp::{
    schemars
};

pub struct Metric {
    value : f64,
    timestamp: u64,    
}

#[derive(Debug, serde::Serialize,schemars::JsonSchema)]
pub struct MetricStats {
    #[schemars(description = "The name of the metric")]
    pub name: String,
    #[schemars(description = "Total sum of the metric values")]
    pub total: f64,
    #[schemars(description = "Average of the metric values")]
    pub average: f64,
    #[schemars(description = "Minimum recorded value for the metric")]
    pub min: f64,
    #[schemars(description = "Maximum recorded value for the metric")]
    pub max: f64,
    #[schemars(description = "Number of recorded values for the metric")]
    pub count: u64,
    #[schemars(description = "Values of the metric over time as (seconds since start of the simulation, value) pairs")]
    pub values_over_time: Vec<(u64, f64)>,
}

pub struct Metrics {
    metrics: HashMap<String, Vec<Metric>>,
}

impl Metrics {
    pub fn new() -> Self {
        Metrics {
            metrics: HashMap::new(),
        }
    }

    pub fn new_from_snapshot(snapshot: &MetricsSnapshot) -> Self {
        let mut metrics = HashMap::new();
        for (name, values) in &snapshot.metrics {
            let metric_values: Vec<Metric> = values.iter().map(|(timestamp, value)| Metric { timestamp: *timestamp, value: *value }).collect();
            metrics.insert(name.clone(), metric_values);
        }

        Metrics {
            metrics,
        }
    }

    pub fn record_metric(&mut self, current_time: u64, name : &str, value: f64) {
        match self.metrics.get_mut(name) {
            Some(metric_list) => {

                // Update existing metric for this timestamp
                if metric_list.last().map_or(false, |m| m.timestamp == current_time) {
                   if let Some(last_metric) = metric_list.last_mut() {
                       last_metric.value += value;
                   }
                   return;
                }

                metric_list.push(Metric {
                    value,
                    timestamp: current_time,
                });
            },
            None => {
                self.metrics.insert(name.to_string(), vec![Metric {
                    value,
                    timestamp: current_time,
                }]);
            }
        }
    }

    pub fn compute_metric_stats(&self, name : &str) -> Option<MetricStats> {
        self.metrics.get(name).map(|values| {
            let total: f64 = values.iter().map(|m| m.value).sum();
            let count = values.len() as f64;
            let average = if count > 0.0 { total / count } else { 0.0 };
            let min = values.iter().map(|m| m.value).fold(f64::INFINITY, f64::min);
            let max = values.iter().map(|m| m.value).fold(f64::NEG_INFINITY, f64::max);
            let values_over_time: Vec<(u64, f64)> = values.iter().map(|m| (m.timestamp, m.value)).collect();

            MetricStats {
                name: name.to_string(),
                total,
                average,
                min,
                max,
                count: count as u64,
                values_over_time,
            }


        })
    }

    pub fn create_snapshot(&self) -> MetricsSnapshot {
        let mut snapshot = HashMap::new();
        for (name, metrics) in &self.metrics {
            let values: Vec<(u64, f64)> = metrics.iter().map(|m| (m.timestamp, m.value)).collect();
            snapshot.insert(name.clone(), values);
        }

        MetricsSnapshot { metrics: snapshot}
    }

    pub fn list_metric_names(&self) -> Vec<String> {
        self.metrics.keys().cloned().collect()
    }
}

