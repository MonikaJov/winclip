// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

#[tokio::test]
async fn items_round_trip_over_dbus() {
    let (_server, daemon) = connect().await;
    let rich = daemon.add_item("rich", "<b>rich</b>").await.unwrap();
    let plain = daemon.add_item("plain", "").await.unwrap();

    let history = daemon.get_history(0).await.unwrap();
    assert_eq!(history.len(), 2);
    assert_eq!(
        (history[0].0, history[0].1.as_str(), history[0].2),
        (plain, "plain", false)
    );
    assert_eq!(
        (history[1].0, history[1].1.as_str(), history[1].2),
        (rich, "rich", true)
    );

    assert_eq!(
        daemon.get_item(rich).await.unwrap(),
        ("rich".into(), "<b>rich</b>".into())
    );
    assert_eq!(
        daemon.get_item(plain).await.unwrap(),
        ("plain".into(), String::new())
    );
    assert_eq!(
        daemon.search("RICH", 0).await.unwrap(),
        vec![history[1].clone()]
    );

    daemon.delete_item(rich).await.unwrap();
    assert_eq!(daemon.get_history(0).await.unwrap().len(), 1);
    daemon.clear_history().await.unwrap();
    assert_eq!(daemon.get_history(0).await.unwrap(), Vec::new());
}
