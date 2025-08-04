#![cfg(test)]
#![allow(missing_docs)]
#![allow(clippy::pedantic)]

use utils_macros::*;

#[test]
fn forward_span() {
    let wrapper = trace_span!("test");
    let bevys = bevy::prelude::trace_span!("test");

    assert_eq!(wrapper.id(), bevys.id());
}

#[test]
fn forward_span_with_fields() {
    let wrapper = trace_span!("test", field = "value");
    let bevys = bevy::prelude::trace_span!("test", field = "value");

    assert_eq!(wrapper.id(), bevys.id());
    assert_eq!(wrapper.metadata(), bevys.metadata())
}

#[test]
fn all_spans_compile() {
    trace_span!("test");
    debug_span!("test");
    info_span!("test");
    warn_span!("test");
    error_span!("test");
}
