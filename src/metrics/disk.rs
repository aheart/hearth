use super::{MetricPlugin, Metrics};
use derive_more::Add;
use serde_derive::Serialize;
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Default, PartialEq, Debug, Clone, Serialize, Add)]
pub struct DiskMetrics {
    write_throughput: f64,
    read_throughput: f64,
}

impl DiskMetrics {
    pub fn divide(self, divisor: f64) -> Self {
        Self {
            write_throughput: self.write_throughput / divisor,
            read_throughput: self.read_throughput / divisor,
        }
    }
}

pub struct DiskMetricPlugin {
    disk: Disk,
    command: String,
}

impl DiskMetricPlugin {
    pub fn new(device: &str) -> Self {
        let disk = Disk::default();
        let command = format!("cat /sys/block/{}/stat", device);
        Self { disk, command }
    }
}

impl MetricPlugin for DiskMetricPlugin {
    fn get_query(&self) -> &str {
        &self.command
    }

    fn process_data(&mut self, raw_data: &str, timestamp: &SystemTime) -> Metrics {
        let disk_stats = DiskStats::from_string(&raw_data, timestamp);

        self.disk.push(disk_stats);

        Metrics::Disk(DiskMetrics {
            write_throughput: self.disk.write_throughput(),
            read_throughput: self.disk.read_throughput(),
        })
    }

    fn empty_metrics(&self) -> Metrics {
        Metrics::Disk(DiskMetrics::default())
    }
}

#[derive(Clone, Copy, Debug)]
pub struct DiskStats {
    reads_completed_successfully: u64,
    reads_merged: u64,
    sectors_read: u64,
    time_spend_reading: u64,
    writes_completed: u64,
    writes_merged: u64,
    sectors_written: u64,
    time_spent_writing: u64,
    ios_currently_in_progress: u64,
    time_spent_doing_ios: u64,
    weighted_time_spent_doing_ios: u64,
    current_time: SystemTime,
}

impl Default for DiskStats {
    fn default() -> DiskStats {
        DiskStats::new(0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, UNIX_EPOCH)
    }
}

impl DiskStats {
    pub fn new(
        reads_completed_successfully: u64,
        reads_merged: u64,
        sectors_read: u64,
        time_spend_reading: u64,
        writes_completed: u64,
        writes_merged: u64,
        sectors_written: u64,
        time_spent_writing: u64,
        ios_currently_in_progress: u64,
        time_spent_doing_ios: u64,
        weighted_time_spent_doing_ios: u64,
        current_time: SystemTime,
    ) -> DiskStats {
        DiskStats {
            reads_completed_successfully,
            reads_merged,
            sectors_read,
            time_spend_reading,
            writes_completed,
            writes_merged,
            sectors_written,
            time_spent_writing,
            ios_currently_in_progress,
            time_spent_doing_ios,
            weighted_time_spent_doing_ios,
            current_time,
        }
    }

    pub fn from_string(raw_data: &str, timestamp: &SystemTime) -> DiskStats {
        let (dist_stats, _): (Vec<&str>, Vec<&str>) =
            raw_data.split(' ').partition(|s| !s.is_empty());

        macro_rules! parse_number {
            ($source:expr, $n:expr) => {
                $source
                    .get($n)
                    .and_then(|v| u64::from_str(v).ok())
                    .unwrap_or(0)
            };
        }

        Self::new(
            parse_number!(dist_stats, 0),
            parse_number!(dist_stats, 1),
            parse_number!(dist_stats, 2),
            parse_number!(dist_stats, 3),
            parse_number!(dist_stats, 4),
            parse_number!(dist_stats, 5),
            parse_number!(dist_stats, 6),
            parse_number!(dist_stats, 7),
            parse_number!(dist_stats, 8),
            parse_number!(dist_stats, 9),
            parse_number!(dist_stats, 10),
            *timestamp,
        )
    }

    pub fn sectors_read(&self) -> u64 {
        self.sectors_read
    }

    pub fn sectors_written(&self) -> u64 {
        self.sectors_written
    }

    pub fn current_time(&self) -> SystemTime {
        self.current_time
    }
}

#[derive(Default)]
pub struct Disk {
    previous_disk_stats: DiskStats,
    read_throughput: f64,
    write_throughput: f64,
}

impl Disk {
    pub fn push(&mut self, disk_stats: DiskStats) {
        macro_rules! diff {
            ($this:expr, $that:expr) => {
                if $this > $that {
                    $this - $that
                } else {
                    $that - $this
                }
            };
        }

        let time_elapsed = disk_stats
            .current_time()
            .duration_since(self.previous_disk_stats.current_time())
            .expect("There is a bug in elapsed time calculation");
        let time_elapsed =
            time_elapsed.as_secs() as f64 + time_elapsed.subsec_millis() as f64 / 1000.0;

        let sectors_read = diff!(
            disk_stats.sectors_read(),
            self.previous_disk_stats.sectors_read()
        ) as f64;

        let sectors_written = diff!(
            disk_stats.sectors_written(),
            self.previous_disk_stats.sectors_written()
        ) as f64;

        self.read_throughput = sectors_read * 512.0 / time_elapsed;
        self.write_throughput = sectors_written * 512.0 / time_elapsed;

        self.previous_disk_stats = disk_stats;
    }

    pub fn read_throughput(&self) -> f64 {
        self.read_throughput
    }

    pub fn write_throughput(&self) -> f64 {
        self.write_throughput
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_process_data() {
        let raw_data_1 = "  255586     4852  7024174   115692    31086    50639  3211504   132760        0    48784   248760";
        let raw_data_2 = "  255600     4852  7027286   115700    31108    50799  3213280   132824        0    48852   248832";
        let read_throughput = (7027286. - 7024174.) * 512.;
        let write_throughput = (3213280. - 3211504.) * 512.;
        assert_parse(raw_data_1, raw_data_2, read_throughput, write_throughput);
        assert_parse("", "", 0.0, 0.0);
    }

    fn assert_parse(
        raw_data_1: &str,
        raw_data_2: &str,
        read_throughput: f64,
        write_throughput: f64,
    ) {
        let mut metric_plugin = DiskMetricPlugin::new("sda");
        let now = UNIX_EPOCH + Duration::new(1531416624, 0);
        println!("{:?}", now);
        metric_plugin.process_data(raw_data_1, &now);
        let now = UNIX_EPOCH + Duration::new(1531416625, 0);
        let metrics = metric_plugin.process_data(raw_data_2, &now);

        let expected_metrics = Metrics::Disk(DiskMetrics {
            read_throughput,
            write_throughput,
        });

        assert_eq!(metrics, expected_metrics);
    }
}
