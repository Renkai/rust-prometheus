// Copyright 2019 TiKV Project Authors. Licensed under Apache-2.0.

#[macro_use]
extern crate lazy_static;
extern crate coarsetime;
extern crate prometheus;
extern crate prometheus_static_metric;

use prometheus::*;

#[allow(unused_imports)]
use prometheus::local::*;
use prometheus::tls::TLSMetricGroup;
use std::collections::HashMap;
use std::thread::LocalKey;
use std::ops::Deref;

#[allow(missing_copy_implementations)]
struct LocalHttpRequestStatisticsInner {
    _foo: LocalIntCounter,
    _bar: LocalIntCounter,
    pub foo: AFLocalIntCounter<'static, LocalHttpRequestStatisticsInner>,
    pub bar: AFLocalIntCounter<'static, LocalHttpRequestStatisticsInner>,
}

pub struct LocalHttpRequestStatistics {
    inner: LocalKey<LocalHttpRequestStatisticsInner>,
}

impl Deref for LocalHttpRequestStatistics {
    type Target = LocalHttpRequestStatisticsInner;
    //TODO: lifetime mismatch
    fn deref(&self) -> &'static Self::Target {
        let res: &'static LocalHttpRequestStatisticsInner = self.inner.with(|m: &'static LocalHttpRequestStatisticsInner| {
            m
        });
        res
    }
}

impl LocalHttpRequestStatisticsInner {
    pub fn from(m: &IntCounterVec) -> LocalHttpRequestStatisticsInner {
        let _foo = m
            .with(&{
                let mut coll = HashMap::new();
                coll.insert("product", "foo");
                coll
            })
            .local();
        let _bar = m
            .with(&{
                let mut coll = HashMap::new();
                coll.insert("product", "bar");
                coll
            })
            .local();

        let mut inner = LocalHttpRequestStatisticsInner {
            _foo,
            _bar,
            foo: AFLocalIntCounter {
                inner: &_foo,
                local_static_group: None,
            },
            bar: AFLocalIntCounter {
                inner: &_bar,
                local_static_group: None,
            },
        };
        inner.foo.set_tls_metric_group(TLSMetricGroup::new(&inner));
        inner.bar.set_tls_metric_group(TLSMetricGroup::new(&inner));
        inner
    }

    pub fn try_get(&self, value: &str) -> Option<&AFLocalIntCounter<LocalHttpRequestStatisticsInner>> {
        match value {
            "foo" => Some(&self.foo),
            "bar" => Some(&self.bar),
            _ => None,
        }
    }
    pub fn flush(&self) {
        self.foo.flush();
        self.bar.flush();
    }
}

impl ::prometheus::local::LocalMetric for LocalHttpRequestStatisticsInner {
    fn flush(&self) {
        LocalHttpRequestStatisticsInner::flush(self);
    }
}

lazy_static! {
pub static ref HTTP_COUNTER_VEC: IntCounterVec =
register_int_counter_vec ! (
"http_requests",
"Total number of HTTP requests.",
& ["product"]    // it doesn't matter for the label order
).unwrap();
}

thread_local! {
static TLS_HTTP_COUNTER_INNER: LocalHttpRequestStatisticsInner = LocalHttpRequestStatisticsInner::from( &HTTP_COUNTER_VEC);
}

pub static TLS_HTTP_COUNTER: LocalHttpRequestStatistics = LocalHttpRequestStatistics {
    inner: TLS_HTTP_COUNTER_INNER,
};

fn main() {
    TLS_HTTP_COUNTER.bar.inc();
}
