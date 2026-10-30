// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

#[test]
fn default_limits() {
    let s = storage();
    assert_eq!(s.max_items().unwrap(), 25);
    assert_eq!(s.max_age_days().unwrap(), 0);
    assert_eq!(s.max_item_bytes().unwrap(), 1024 * 1024);
}

#[test]
fn count_limit_keeps_newest() {
    let mut s = storage();
    s.set_max_items(3).unwrap();
    for t in ["1", "2", "3", "4", "5"] {
        s.add_item(t, None).unwrap();
    }
    assert_eq!(previews(&s.get_history(0).unwrap()), ["5", "4", "3"]);
}

#[test]
fn default_count_limit_is_enforced() {
    let mut s = storage();
    for i in 0..30 {
        s.add_item(&i.to_string(), None).unwrap();
    }
    assert_eq!(s.get_history(0).unwrap().len(), 25);
}

#[test]
fn lowering_max_items_trims_immediately() {
    let mut s = storage();
    for t in ["1", "2", "3", "4"] {
        s.add_item(t, None).unwrap();
    }
    assert_eq!(s.set_max_items(2).unwrap(), 2);
    assert_eq!(previews(&s.get_history(0).unwrap()), ["4", "3"]);
}

#[test]
fn age_limit() {
    let (mut s, time) = storage_with_clock();
    s.add_item("old", None).unwrap();

    time.store(T0 + 2 * SECONDS_PER_DAY, Ordering::SeqCst);
    assert_eq!(s.trim().unwrap(), 0, "no age limit by default");

    s.add_item("new", None).unwrap();
    assert_eq!(s.set_max_age_days(1).unwrap(), 1);
    assert_eq!(previews(&s.get_history(0).unwrap()), ["new"]);

    time.store(T0 + 3 * SECONDS_PER_DAY, Ordering::SeqCst);
    assert_eq!(s.trim().unwrap(), 0, "exactly one day old is kept");
    time.store(T0 + 3 * SECONDS_PER_DAY + 1, Ordering::SeqCst);
    assert_eq!(s.trim().unwrap(), 1);
}

#[test]
fn lowering_max_item_bytes_removes_bigger_items() {
    let mut s = storage();
    s.add_item(&"b".repeat(2000), None).unwrap();
    s.add_item("small", Some(&"h".repeat(1100))).unwrap();
    s.add_item("tiny", None).unwrap();
    assert_eq!(s.set_max_item_bytes(1024).unwrap(), 2);
    assert_eq!(previews(&s.get_history(0).unwrap()), ["tiny"]);
}

#[test]
fn limits_only_accept_values_in_range() {
    let mut s = storage();
    for result in [
        s.set_max_items(0),
        s.set_max_items(1001),
        s.set_max_age_days(3651),
        s.set_max_item_bytes(1023),
        s.set_max_item_bytes(10 * 1024 * 1024 + 1),
    ] {
        assert!(matches!(result, Err(Error::InvalidArgument(_))));
    }
    assert_eq!(s.max_items().unwrap(), 25);
    assert_eq!(s.max_age_days().unwrap(), 0);
    assert_eq!(s.max_item_bytes().unwrap(), 1024 * 1024);

    for result in [
        s.set_max_items(1),
        s.set_max_items(1000),
        s.set_max_age_days(3650),
        s.set_max_item_bytes(1024),
        s.set_max_item_bytes(10 * 1024 * 1024),
    ] {
        assert!(result.is_ok());
    }
}
