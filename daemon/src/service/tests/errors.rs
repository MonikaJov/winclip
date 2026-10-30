// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

#[tokio::test]
async fn storage_errors_become_named_dbus_errors() {
    let (_server, daemon) = connect().await;

    let not_found = daemon.get_item(42).await.unwrap_err();
    assert_eq!(error_name(&not_found), "org.winclip.Daemon1.Error.NotFound");

    let empty = daemon.add_item("", "").await.unwrap_err();
    assert_eq!(
        error_name(&empty),
        "org.winclip.Daemon1.Error.InvalidArgument"
    );

    daemon.set_max_item_bytes(1024).await.unwrap();
    let too_large = daemon.add_item(&"x".repeat(1025), "").await.unwrap_err();
    assert_eq!(error_name(&too_large), "org.winclip.Daemon1.Error.TooLarge");
}
