use super::MetricPlugin;
use std::collections::HashMap;
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct DiskMetricPlugin {
    disk: Disk,
}

impl MetricPlugin for DiskMetricPlugin {
    fn new() -> Self {
        let disk = Disk::default();
        Self { disk }
    }

    fn get_query(&self) -> &'static str {
        "cat /sys/block/sda/stat"
    }

    fn process_data(&mut self, raw_data: &str) -> HashMap<String, String> {
        let disk_stats = DiskStats::from_string(&raw_data);

        self.disk.push(disk_stats);

        let write_throughput = format!("{}", self.disk.write_throughput());
        let read_throughput = format!("{}", self.disk.read_throughput());

        let mut metrics = HashMap::new();
        metrics.insert("write_throughput".into(), write_throughput);
        metrics.insert("read_throughput".into(), read_throughput);
        metrics
    }

    fn empty_metrics(&self) -> HashMap<String, String> {
        let mut metrics = HashMap::new();
        metrics.insert("write_throughput".into(), "0".into());
        metrics.insert("read_throughput".into(), "0".into());
        metrics
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
        DiskStats::new(
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            UNIX_EPOCH,
        )
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

    pub fn from_string(raw_data: &str) -> DiskStats {
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
            SystemTime::now(),
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
            .unwrap();
        let time_elapsed =
            (time_elapsed.as_secs() as f64 * 1000.0) + time_elapsed.subsec_millis() as f64;
        let time_elapsed = time_elapsed / 1000.0;

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
