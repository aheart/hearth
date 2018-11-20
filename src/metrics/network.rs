use super::MetricPlugin;
use std::collections::HashMap;
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct NetworkMetricPlugin {
    network: Network,
    command: String,
}

impl NetworkMetricPlugin {
    pub fn new(interface: &str) -> Self {
        let network = Network::default();
        let command = format!("cat /sys/class/net/{0}/statistics/rx_bytes /sys/class/net/{0}/statistics/tx_bytes", interface);
        Self { network, command }
    }
}

impl MetricPlugin for NetworkMetricPlugin {
    fn get_query(&self) -> &str {
        &self.command
    }

    fn process_data(&mut self, raw_data: &str, timestamp: &SystemTime) -> HashMap<String, String> {
        let network_stats = NetworkStats::from_string(&raw_data, timestamp);

        self.network.push(network_stats);

        let up_bandwidth = format!("{}", self.network.up_bandwidth());
        let down_bandwidth = format!("{}", self.network.down_bandwidth());

        let mut metrics = HashMap::new();
        metrics.insert("up_bandwidth".into(), up_bandwidth);
        metrics.insert("down_bandwidth".into(), down_bandwidth);
        metrics
    }

    fn empty_metrics(&self) -> HashMap<String, String> {
        let mut metrics = HashMap::new();
        metrics.insert("up_bandwidth".into(), "0".into());
        metrics.insert("down_bandwidth".into(), "0".into());
        metrics
    }
}

#[derive(Clone, Copy, Debug)]
pub struct NetworkStats {
    rx_bytes: u64,
    tx_bytes: u64,
    current_time: SystemTime,
}

impl Default for NetworkStats {
    fn default() -> Self {
        Self::new(
            0,
            0,
            UNIX_EPOCH
        )
    }
}

impl NetworkStats {
    pub fn new(
        rx_bytes: u64,
        tx_bytes: u64,
        current_time: SystemTime,
    ) -> Self {
        Self {
            rx_bytes,
            tx_bytes,
            current_time,
        }
    }

    pub fn from_string(raw_data: &str, timestamp: &SystemTime) -> Self {
        let (dist_stats, _): (Vec<&str>, Vec<&str>) =
            raw_data.split('\n').partition(|s| !s.is_empty());

        macro_rules! parse_number {
            ($source:expr, $n:expr) => {
                $source
                    .get($n)
                    .and_then(|v| u64::from_str(v).ok())
                    .unwrap_or(0)
            };
        };

        Self::new(
            parse_number!(dist_stats, 0),
            parse_number!(dist_stats, 1),
            timestamp.clone(),
        )
    }

    pub fn rx_bytes(&self) -> u64 {
        self.rx_bytes
    }

    pub fn tx_bytes(&self) -> u64 {
        self.tx_bytes
    }

    pub fn current_time(&self) -> SystemTime {
        self.current_time
    }
}

#[derive(Default)]
pub struct Network {
    previous_network_stats: NetworkStats,
    down_bandwidth: f64,
    up_bandwidth: f64,
}

impl Network {
    pub fn push(&mut self, network_stats: NetworkStats) {
        macro_rules! diff {
            ($this:expr, $that:expr) => {
                if $this > $that {
                    $this - $that
                } else {
                    $that - $this
                }
            };
        }

        let time_elapsed = network_stats
            .current_time()
            .duration_since(self.previous_network_stats.current_time())
            .unwrap();
        let time_elapsed =
            time_elapsed.as_secs() as f64 + time_elapsed.subsec_millis() as f64 / 1000.0 ;

        let rx_bytes = diff!(
            network_stats.rx_bytes(),
            self.previous_network_stats.rx_bytes()
        ) as f64;

        let tx_bytes = diff!(
            network_stats.tx_bytes(),
            self.previous_network_stats.tx_bytes()
        ) as f64;

        self.down_bandwidth = rx_bytes / time_elapsed;
        self.up_bandwidth = tx_bytes / time_elapsed;
        self.previous_network_stats = network_stats;
    }

    pub fn down_bandwidth(&self) -> f64 {
        self.down_bandwidth
    }

    pub fn up_bandwidth(&self) -> f64 {
        self.up_bandwidth
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_process_data() {
        let raw_data_1 = "33597756273\n11137558032";
        let raw_data_2 = "33597768357\n11137566224";

        let down_bandwidth: i64 = 33597768357 - 33597756273;
        let up_bandwidth: i64 = 11137566224 - 11137558032;
        assert_parse(raw_data_1, raw_data_2, &down_bandwidth.to_string(), &up_bandwidth.to_string());
        assert_parse("", "", "0", "0");
    }

    fn assert_parse(raw_data_1: &str, raw_data_2: &str, down_bandwidth: &str, up_bandwidth: &str) {
        let mut metric_plugin = NetworkMetricPlugin::new("eth0");
        let now = UNIX_EPOCH + Duration::new(1531416624, 0);
        println!("{:?}", now);
        metric_plugin.process_data(raw_data_1, &now);
        let now = UNIX_EPOCH + Duration::new(1531416625, 0);
        let metrics = metric_plugin.process_data(raw_data_2, &now);

        let mut expected_metrics = HashMap::new();
        expected_metrics.insert("down_bandwidth".to_string(), down_bandwidth.to_string());
        expected_metrics.insert("up_bandwidth".to_string(), up_bandwidth.to_string());

        assert_eq!(metrics, expected_metrics);
    }
}
