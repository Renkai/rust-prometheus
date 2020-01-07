// Copyright 2019 TiKV Project Authors. Licensed under Apache-2.0.

use crate::local::{LocalHistogram, LocalMetric};
use crate::Histogram;
use coarsetime::{Instant, Updater};
use std::cell::Cell;
use std::ops::Deref;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::LocalKey;

lazy_static! {
    static ref UPDATER_IS_RUNNING: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
}

fn ensure_updater() {
    if !UPDATER_IS_RUNNING.compare_and_swap(false, true, Ordering::SeqCst) {
        Updater::new(200).start().unwrap();
    }
}
/// use for auto flush thread local storage
#[derive(Debug)]
pub struct TLSMetricGroup<'a, T: LocalMetric> {
    inner: &'a T,
    last_flush_time: Cell<Instant>,
}

impl<'a, 'b, T: LocalMetric> TLSMetricGroup<'b, T>
where
    'a: 'b,
{
    ///new a TLSMetricGroup
    pub fn new(inner: &'a T) -> Self {
        ensure_updater();
        Self {
            inner,
            last_flush_time: Cell::new(Instant::recent()),
        }
    }

    /// Flushes the inner metrics if at least 1 second is passed since last flush.
    pub fn may_flush_all(&self) -> bool {
        let recent = Instant::recent();
        if (recent - self.last_flush_time.get()).as_secs() == 0 {
            // Minimum flush interval is 1s
            return false;
        }
        self.inner.flush();
        self.last_flush_time.set(recent);
        true
    }

    /// Flushes the inner metrics immediately.
    pub fn force_flush_all(&self) {
        self.inner.flush()
    }
}

impl<T: LocalMetric> Deref for TLSMetricGroup<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.inner
    }
}
