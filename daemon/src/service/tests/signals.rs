// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

#[tokio::test]
async fn changes_emit_history_changed() {
    let (_server, daemon) = connect().await;
    let mut signals = daemon.receive_history_changed().await.unwrap();

    let id = daemon.add_item("a", "").await.unwrap();
    next_signal(&mut signals).await;

    daemon.delete_item(id).await.unwrap();
    next_signal(&mut signals).await;

    daemon.add_item("b", "").await.unwrap();
    daemon.add_item("c", "").await.unwrap();
    next_signal(&mut signals).await;
    next_signal(&mut signals).await;

    daemon.set_max_items(1).await.unwrap();
    next_signal(&mut signals).await;

    daemon.clear_history().await.unwrap();
    next_signal(&mut signals).await;
}

#[tokio::test]
async fn periodic_trim_runs_against_the_served_daemon() {
    let (server, daemon) = connect().await;
    daemon.add_item("a", "").await.unwrap();
    crate::service::trim(&server).await.unwrap();
    assert_eq!(daemon.get_history(0).await.unwrap().len(), 1);
}
