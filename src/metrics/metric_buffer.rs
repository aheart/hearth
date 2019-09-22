use crate::metrics::aggregator::NodeMetrics;
use std::collections::HashMap;

pub struct MetricBuffer {
    limit: usize,
    storage: Vec<NodeMetrics>,
}

impl MetricBuffer {
    pub fn new(limit: usize) -> MetricBuffer {
        MetricBuffer {
            limit,
            storage: Vec::new(),
        }
    }

    pub fn storage(&self) -> &Vec<NodeMetrics> {
        &self.storage
    }

    pub fn push(&mut self, metrics: NodeMetrics) {
        let length = self.storage.len();
        if length >= self.limit {
            self.storage.drain(0..(length - self.limit));
        }

        self.storage.push(metrics);
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
