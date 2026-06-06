// SPDX-License-Identifier: GPL-3.0-or-later
// DBus interface for the Soulless applet.

use cosmic::iced::Subscription;
use futures_util::StreamExt;

#[derive(Debug, Clone)]
pub enum DbusMessage {
    Activate,
}

struct SoullessInterface {
    sender: tokio::sync::mpsc::Sender<DbusMessage>,
}

#[zbus::interface(name = "com.github.hmrdsmoke.SoullessApplet")]
impl SoullessInterface {
    async fn activate(&self) {
        eprintln!("DBus: activate called!");
        let _ = self.sender.send(DbusMessage::Activate).await;
    }
}

pub fn subscription() -> Subscription<DbusMessage> {
    Subscription::run(|| {
        futures_util::stream::unfold(
            Step::Init,
            |step| async move {
                match step {
                    Step::Init => {
                        let (tx, rx) = tokio::sync::mpsc::channel::<DbusMessage>(10);
                        let interface = SoullessInterface { sender: tx };
                        match zbus::connection::Builder::session()
                            .unwrap()
                            .name("com.github.hmrdsmoke.SoullessApplet")
                            .unwrap()
                            .serve_at("/com/github/hmrdsmoke/SoullessApplet", interface)
                            .unwrap()
                            .build()
                            .await
                        {
                            Ok(conn) => {
                                Some((None, Step::Running { _conn: conn, rx }))
                            }
                            Err(e) => {
                                eprintln!("DBus error: {e}");
                                Some((None, Step::Dead))
                            }
                        }
                    }
                    Step::Running { _conn, mut rx } => {
                        match rx.recv().await {
                            Some(msg) => {
                                Some((Some(msg), Step::Running { _conn, rx }))
                            }
                            None => None,
                        }
                    }
                    Step::Dead => {
                        futures_util::future::pending::<()>().await;
                        None
                    }
                }
            },
        )
        .filter_map(|x| async move { x })
    })
}

enum Step {
    Init,
    Running {
        _conn: zbus::Connection,
        rx: tokio::sync::mpsc::Receiver<DbusMessage>,
    },
    Dead,
}
