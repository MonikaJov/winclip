// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

#[tokio::test]
async fn limits_are_readable_and_writable_properties() {
    let (_server, daemon) = connect().await;
    assert_eq!(daemon.max_items().await.unwrap(), 25);
    assert_eq!(daemon.max_age_days().await.unwrap(), 0);
    assert_eq!(daemon.max_item_bytes().await.unwrap(), 1024 * 1024);

    daemon.set_max_items(3).await.unwrap();
    daemon.set_max_age_days(7).await.unwrap();
    daemon.set_max_item_bytes(2048).await.unwrap();
    assert_eq!(daemon.max_items().await.unwrap(), 3);
    assert_eq!(daemon.max_age_days().await.unwrap(), 7);
    assert_eq!(daemon.max_item_bytes().await.unwrap(), 2048);
    assert_eq!(daemon.version().await.unwrap(), env!("CARGO_PKG_VERSION"));
}

#[tokio::test]
async fn out_of_range_limit_is_invalid_args() {
    let (_server, daemon) = connect().await;
    let err = daemon.set_max_items(0).await.unwrap_err();
    assert_eq!(error_name(&err), "org.freedesktop.DBus.Error.InvalidArgs");
    assert_eq!(daemon.max_items().await.unwrap(), 25);
}
