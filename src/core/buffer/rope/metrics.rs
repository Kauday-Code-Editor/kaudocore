/// metrics.rs
/// File for tracking various metrics related to the rope data structure.
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, error, info, instrument, trace, warn};

/// Struct to hold various metrics related to the rope data structure.
#[derive(Debug, Default)]
pub struct RopeMetrics {
    pub total_nodes: AtomicUsize,
    pub total_leaves: AtomicUsize,
    pub total_internal_nodes: AtomicUsize,
    pub max_height: AtomicUsize,
    pub total_splits: AtomicUsize,
    pub total_merges: AtomicUsize,
    pub total_rebalances: AtomicUsize,
    pub total_insertions: AtomicUsize,
    pub total_deletions: AtomicUsize,
    pub total_searches: AtomicUsize,
    pub total_rotations: AtomicUsize,
    pub total_bytes: AtomicUsize,
    pub total_operations: AtomicUsize,
    pub total_time_nanos: AtomicU64,
    pub peak_memory_usage: AtomicUsize,
    pub cache_hits: AtomicUsize,
    pub cache_misses: AtomicUsize,
}

impl RopeMetrics {
    pub fn new() -> Self {
        Self {
            total_nodes: AtomicUsize::new(0),
            total_leaves: AtomicUsize::new(0),
            total_internal_nodes: AtomicUsize::new(0),
            max_height: AtomicUsize::new(0),
            total_splits: AtomicUsize::new(0),
            total_merges: AtomicUsize::new(0),
            total_rebalances: AtomicUsize::new(0),
            total_insertions: AtomicUsize::new(0),
            total_deletions: AtomicUsize::new(0),
            total_searches: AtomicUsize::new(0),
            total_rotations: AtomicUsize::new(0),
            total_bytes: AtomicUsize::new(0),
            total_operations: AtomicUsize::new(0),
            total_time_nanos: AtomicU64::new(0),
            peak_memory_usage: AtomicUsize::new(0),
            cache_hits: AtomicUsize::new(0),
            cache_misses: AtomicUsize::new(0),
        }
    }

    #[inline]
    pub fn record_insertion(&self, bytes: usize) {
        self.total_insertions.fetch_add(1, Ordering::Relaxed);
        self.total_bytes.fetch_add(bytes, Ordering::Relaxed);
        self.total_operations.fetch_add(1, Ordering::Relaxed);
    }

    #[inline]
    pub fn record_deletion(&self, bytes: usize) {
        self.total_deletions.fetch_add(1, Ordering::Relaxed);
        self.total_bytes.fetch_sub(bytes, Ordering::Relaxed);
        self.total_operations.fetch_add(1, Ordering::Relaxed);
    }

    #[inline]
    pub fn record_search(&self) {
        self.total_searches.fetch_add(1, Ordering::Relaxed);
        self.total_operations.fetch_add(1, Ordering::Relaxed);
    }

    #[inline]
    pub fn record_split(&self) {
        self.total_splits.fetch_add(1, Ordering::Relaxed);
    }

    #[inline]
    pub fn record_merge(&self) {
        self.total_merges.fetch_add(1, Ordering::Relaxed);
    }

    #[inline]
    pub fn record_rebalance(&self) {
        self.total_rebalances.fetch_add(1, Ordering::Relaxed);
    }

    #[inline]
    pub fn record_rotation(&self) {
        self.total_rotations.fetch_add(1, Ordering::Relaxed);
    }

    #[inline]
    pub fn record_time(&self, duration: Duration) {
        self.total_time_nanos
            .fetch_add(duration.as_nanos() as u64, Ordering::Relaxed);
    }

    #[inline]
    pub fn record_node(&self, is_leaf: bool) {
        self.total_nodes.fetch_add(1, Ordering::Relaxed);
        if is_leaf {
            self.total_leaves.fetch_add(1, Ordering::Relaxed);
        } else {
            self.total_internal_nodes.fetch_add(1, Ordering::Relaxed);
        }
    }

    #[inline]
    pub fn record_node_removed(&self, is_leaf: bool) {
        self.total_nodes.fetch_sub(1, Ordering::Relaxed);
        if is_leaf {
            self.total_leaves.fetch_sub(1, Ordering::Relaxed);
        } else {
            self.total_internal_nodes.fetch_sub(1, Ordering::Relaxed);
        }
    }

    #[inline]
    pub fn update_max_height(&self, height: usize) {
        self.max_height.fetch_max(height, Ordering::Relaxed);
    }

    #[inline]
    pub fn record_memory_usage(&self, bytes: usize) {
        self.peak_memory_usage.fetch_max(bytes, Ordering::Relaxed);
    }

    #[inline]
    pub fn record_cache_hit(&self) {
        self.cache_hits.fetch_add(1, Ordering::Relaxed);
    }

    #[inline]
    pub fn record_cache_miss(&self) {
        self.cache_misses.fetch_add(1, Ordering::Relaxed);
    }

    pub fn get_total_time(&self) -> Duration {
        Duration::from_nanos(self.total_time_nanos.load(Ordering::Relaxed))
    }

    pub fn get_cache_hit_ratio(&self) -> f64 {
        let hits = self.cache_hits.load(Ordering::Relaxed) as f64;
        let misses = self.cache_misses.load(Ordering::Relaxed) as f64;
        let total = hits + misses;
        if total > 0.0 {
            hits / total
        } else {
            0.0
        }
    }

    pub fn get_operations_per_second(&self) -> f64 {
        let ops = self.total_operations.load(Ordering::Relaxed) as f64;
        let time_secs = self.get_total_time().as_secs_f64();
        if time_secs > 0.0 {
            ops / time_secs
        } else {
            0.0
        }
    }

    pub fn reset(&self) {
        self.total_nodes.store(0, Ordering::Relaxed);
        self.total_leaves.store(0, Ordering::Relaxed);
        self.total_internal_nodes.store(0, Ordering::Relaxed);
        self.max_height.store(0, Ordering::Relaxed);
        self.total_splits.store(0, Ordering::Relaxed);
        self.total_merges.store(0, Ordering::Relaxed);
        self.total_rebalances.store(0, Ordering::Relaxed);
        self.total_insertions.store(0, Ordering::Relaxed);
        self.total_deletions.store(0, Ordering::Relaxed);
        self.total_searches.store(0, Ordering::Relaxed);
        self.total_rotations.store(0, Ordering::Relaxed);
        self.total_bytes.store(0, Ordering::Relaxed);
        self.total_operations.store(0, Ordering::Relaxed);
        self.total_time_nanos.store(0, Ordering::Relaxed);
        self.peak_memory_usage.store(0, Ordering::Relaxed);
        self.cache_hits.store(0, Ordering::Relaxed);
        self.cache_misses.store(0, Ordering::Relaxed);
    }
}

impl Clone for RopeMetrics {
    fn clone(&self) -> Self {
        Self {
            total_nodes: AtomicUsize::new(self.total_nodes.load(Ordering::Relaxed)),
            total_leaves: AtomicUsize::new(self.total_leaves.load(Ordering::Relaxed)),
            total_internal_nodes: AtomicUsize::new(self.total_internal_nodes.load(Ordering::Relaxed)),
            max_height: AtomicUsize::new(self.max_height.load(Ordering::Relaxed)),
            total_splits: AtomicUsize::new(self.total_splits.load(Ordering::Relaxed)),
            total_merges: AtomicUsize::new(self.total_merges.load(Ordering::Relaxed)),
            total_rebalances: AtomicUsize::new(self.total_rebalances.load(Ordering::Relaxed)),
            total_insertions: AtomicUsize::new(self.total_insertions.load(Ordering::Relaxed)),
            total_deletions: AtomicUsize::new(self.total_deletions.load(Ordering::Relaxed)),
            total_searches: AtomicUsize::new(self.total_searches.load(Ordering::Relaxed)),
            total_rotations: AtomicUsize::new(self.total_rotations.load(Ordering::Relaxed)),
            total_bytes: AtomicUsize::new(self.total_bytes.load(Ordering::Relaxed)),
            total_operations: AtomicUsize::new(self.total_operations.load(Ordering::Relaxed)),
            total_time_nanos: AtomicU64::new(self.total_time_nanos.load(Ordering::Relaxed)),
            peak_memory_usage: AtomicUsize::new(self.peak_memory_usage.load(Ordering::Relaxed)),
            cache_hits: AtomicUsize::new(self.cache_hits.load(Ordering::Relaxed)),
            cache_misses: AtomicUsize::new(self.cache_misses.load(Ordering::Relaxed)),
        }
    }
}

impl std::fmt::Display for RopeMetrics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "RopeMetrics {{\n  \
            total_nodes: {}, total_leaves: {}, total_internal_nodes: {}, max_height: {},\n  \
            total_splits: {}, total_merges: {}, total_rebalances: {}, total_rotations: {},\n  \
            total_insertions: {}, total_deletions: {}, total_searches: {},\n  \
            total_bytes: {}, total_operations: {}, total_time: {:?},\n  \
            peak_memory_usage: {} bytes, cache_hit_ratio: {:.2}%,\n  \
            operations_per_second: {:.2}\n}}",
            self.total_nodes.load(Ordering::Relaxed),
            self.total_leaves.load(Ordering::Relaxed),
            self.total_internal_nodes.load(Ordering::Relaxed),
            self.max_height.load(Ordering::Relaxed),
            self.total_splits.load(Ordering::Relaxed),
            self.total_merges.load(Ordering::Relaxed),
            self.total_rebalances.load(Ordering::Relaxed),
            self.total_rotations.load(Ordering::Relaxed),
            self.total_insertions.load(Ordering::Relaxed),
            self.total_deletions.load(Ordering::Relaxed),
            self.total_searches.load(Ordering::Relaxed),
            self.total_bytes.load(Ordering::Relaxed),
            self.total_operations.load(Ordering::Relaxed),
            self.get_total_time(),
            self.peak_memory_usage.load(Ordering::Relaxed),
            self.get_cache_hit_ratio() * 100.0,
            self.get_operations_per_second()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_metrics_recording() {
        let metrics = RopeMetrics::new();
        metrics.record_insertion(100);
        metrics.record_deletion(50);
        metrics.record_search();
        metrics.record_split();
        metrics.record_merge();
        metrics.record_rebalance();
        metrics.record_rotation();
        metrics.record_node(true);
        metrics.record_node(false);
        metrics.record_time(Duration::from_millis(10));
        metrics.update_max_height(5);
        metrics.record_memory_usage(1024);
        metrics.record_cache_hit();
        metrics.record_cache_miss();

        assert_eq!(metrics.total_insertions.load(Ordering::Relaxed), 1);
        assert_eq!(metrics.total_deletions.load(Ordering::Relaxed), 1);
        assert_eq!(metrics.total_searches.load(Ordering::Relaxed), 1);
        assert_eq!(metrics.total_nodes.load(Ordering::Relaxed), 2);
        assert_eq!(metrics.total_leaves.load(Ordering::Relaxed), 1);
        assert_eq!(metrics.total_internal_nodes.load(Ordering::Relaxed), 1);
        assert_eq!(metrics.max_height.load(Ordering::Relaxed), 5);
        assert!(metrics.get_total_time() >= Duration::from_millis(10));
        assert!(metrics.get_cache_hit_ratio() > 0.0);
    }

    #[test]
    fn test_metrics_reset() {
        let metrics = RopeMetrics::new();
        metrics.record_insertion(100);
        metrics.record_search();
        metrics.reset();
        
        assert_eq!(metrics.total_insertions.load(Ordering::Relaxed), 0);
        assert_eq!(metrics.total_searches.load(Ordering::Relaxed), 0);
        assert_eq!(metrics.total_operations.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn test_metrics_clone() {
        let metrics = RopeMetrics::new();
        metrics.record_insertion(100);
        
        let cloned = metrics.clone();
        assert_eq!(
            cloned.total_insertions.load(Ordering::Relaxed),
            metrics.total_insertions.load(Ordering::Relaxed)
        );
    }
}