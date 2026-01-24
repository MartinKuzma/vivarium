use std::collections::HashMap;
use rmcp::{
    schemars
};

pub struct Metric {
    value : f64,
    timestamp: u64,    
}

#[derive(Debug, serde::Serialize,schemars::JsonSchema)]
pub struct MetricStats {
    pub name: String,
    pub total: f64,
    pub average: f64,
    pub min: f64,
    pub max: f64,
    #[schemars(description = "Number of recorded values for the metric")]
    pub count: u64,
    #[schemars(description = "Values of the metric over time as (seconds since start of the simulation, value) pairs")]
    pub values_over_time: Vec<(u64, f64)>,
}

pub struct Metrics {
    current_time: u64,
    metrics: HashMap<String, Vec<Metric>>,
}

impl Metrics {
    pub fn new(start_time: u64) -> Self {
        Metrics {
            current_time: start_time,
            metrics: HashMap::new(),
        }
    }

    pub fn update_time(&mut self, new_time: u64) {
        self.current_time = new_time;
    }

    pub fn record_metric(&mut self, name : &str, value: f64) {
        match self.metrics.get_mut(name) {
            Some(metric_list) => {

                // Update existing metric for this timestamp
                if metric_list.last().map_or(false, |m| m.timestamp == self.current_time) {
                   if let Some(last_metric) = metric_list.last_mut() {
                       last_metric.value += value;
                   }
                   return;
                }

                metric_list.push(Metric {
                    value,
                    timestamp: self.current_time,
                });
            },
            None => {
                self.metrics.insert(name.to_string(), vec![Metric {
                    value,
                    timestamp: self.current_time,
                }]);
            }
        }
    }

    pub fn get_metric_stats(&self, name : &str) -> Option<MetricStats> {
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

    pub fn get_all_metrics(&self) -> Vec<MetricStats> {
        let mut all_stats = Vec::new();
        for (name, _) in &self.metrics {
            if let Some(stats) = self.get_metric_stats(name) {
                all_stats.push(stats);
            }
        }
        all_stats
    }
}

