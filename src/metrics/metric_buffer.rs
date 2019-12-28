use crate::metrics::aggregator::NodeMetrics;
use crate::ws::server::View;
use std::collections::HashMap;

pub struct MetricBuffer {
    limit: usize,
    storage_1s: Vec<NodeMetrics>,

    samples_since_5s_rollup: u8,
    storage_5s: Vec<NodeMetrics>,

    samples_since_15s_rollup: u8,
    storage_15s: Vec<NodeMetrics>,
}

impl MetricBuffer {
    pub fn new(limit: usize) -> MetricBuffer {
        MetricBuffer {
            limit,
            storage_1s: Vec::with_capacity(limit),
            samples_since_5s_rollup: 0,
            storage_5s: Vec::with_capacity(limit),
            samples_since_15s_rollup: 0,
            storage_15s: Vec::with_capacity(limit),
        }
    }

    pub fn storage(&self, timeframe: View) -> &Vec<NodeMetrics> {
        match timeframe {
            View::OverviewOneSecond => &self.storage_1s,
            View::OverviewFiveSeconds => &self.storage_5s,
            View::OverviewFifteenSeconds => &self.storage_15s,
        }
    }

    pub fn push(&mut self, metrics: NodeMetrics) {
        let length = self.storage_1s.len();
        if length >= self.limit {
            self.storage_1s.drain(0..(length - self.limit));
        }

        self.storage_1s.push(metrics);
        self.samples_since_5s_rollup += 1;
        self.samples_since_15s_rollup += 1;

        // 10m rollup
        if self.samples_since_5s_rollup == 5 {
            let metrics: Vec<NodeMetrics> = self.storage_1s.iter().cloned().rev().take(5).collect();
            let rollup = NodeMetrics::aggregate_avg(metrics);
            let length = self.storage_5s.len();
            if length >= self.limit {
                self.storage_5s.drain(0..(length - self.limit));
            }
            self.storage_5s.push(rollup);
            self.samples_since_5s_rollup = 0;
        }

        // 30m Rollup
        if self.samples_since_15s_rollup == 15 {
            let metrics: Vec<NodeMetrics> = self.storage_5s.iter().cloned().rev().take(3).collect();
            let rollup = NodeMetrics::aggregate_avg(metrics);
            let length = self.storage_15s.len();
            if length >= self.limit {
                self.storage_15s.drain(0..(length - self.limit));
            }
            self.storage_15s.push(rollup);
            self.samples_since_15s_rollup = 0;
        }
    }
}

pub struct MetricBufferMap {
    limit: usize,
    storage: HashMap<String, MetricBuffer>,
}

impl MetricBufferMap {
    pub fn new(limit: usize) -> MetricBufferMap {
        MetricBufferMap {
            limit,
            storage: HashMap::new(),
        }
    }

    pub fn storage(&self) -> &HashMap<String, MetricBuffer> {
        &self.storage
    }

    pub fn push(&mut self, key: &str, metrics: NodeMetrics) {
        if let Some(buffer) = self.storage.get_mut(key) {
            buffer.push(metrics);
        } else {
            let key = key.to_string();
            let mut buffer = MetricBuffer::new(self.limit);
            buffer.push(metrics);
            self.storage.insert(key, buffer);
        }
    }
}
