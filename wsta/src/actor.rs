// trade/wsta/src/actor.rs
//
// The wsta actor is the replacement for the Iced app's implicit frontend state.
// It owns selected view, browser sessions, debug output, and the seek bridge.
//
// This is intentionally core_front_main-like:
//   browser input -> actor -> seek bridge -> dsta mesh
//   dsta mesh -> seek bridge -> actor -> browser sessions

use crate::protocol::{BrowserToWsta, WstaToBrowser, WstaView};
use crate::seek_bridge::{SeekBridgeCommand, SeekBridgeEvent};
use tokio::sync::{broadcast, mpsc};

#[derive(Debug)]
pub enum ActorMsg {
    BrowserPacket(BrowserToWsta),
    SeekEvent(SeekBridgeEvent),
    BrowserConnected,
}

#[derive(Clone)]
pub struct ActorHandle {
    tx: mpsc::Sender<ActorMsg>,
    browser_tx: broadcast::Sender<WstaToBrowser>,
}

impl ActorHandle {
    pub async fn send_browser_packet(&self, pkt: BrowserToWsta) -> Result<(), String> {
        self.tx
            .send(ActorMsg::BrowserPacket(pkt))
            .await
            .map_err(|e| format!("wsta actor is down: {}", e))
    }

    pub async fn notify_browser_connected(&self) {
        let _ = self.tx.send(ActorMsg::BrowserConnected).await;
    }

    pub fn subscribe_browser_events(&self) -> broadcast::Receiver<WstaToBrowser> {
        self.browser_tx.subscribe()
    }
}

pub fn spawn_actor(seek_backend_addr: String) -> ActorHandle {
    let (actor_tx, mut actor_rx) = mpsc::channel::<ActorMsg>(8192);
    let (browser_tx, _) = broadcast::channel::<WstaToBrowser>(8192);

    let (seek_cmd_tx, seek_event_rx) = crate::seek_bridge::spawn_seek_bridge(seek_backend_addr);

    {
        let actor_tx = actor_tx.clone();
        tokio::spawn(async move {
            let mut rx = seek_event_rx;
            while let Some(ev) = rx.recv().await {
                if actor_tx.send(ActorMsg::SeekEvent(ev)).await.is_err() {
                    break;
                }
            }
        });
    }

    {
        let browser_tx = browser_tx.clone();

        tokio::spawn(async move {
            let mut selected_view = WstaView::DrR;

            let _ = browser_tx.send(WstaToBrowser::DebugLine {
                line: "wsta actor started".to_string(),
            });

            loop {
                tokio::select! {
                    Some(msg) = actor_rx.recv() => {
                        match msg {
                            ActorMsg::BrowserConnected => {
                                let _ = browser_tx.send(WstaToBrowser::Connected {
                                    message: "browser session connected to wsta actor".to_string(),
                                });
                                let _ = browser_tx.send(WstaToBrowser::SelectedView {
                                    view: selected_view,
                                });
                            }

                            ActorMsg::BrowserPacket(pkt) => {
                                match &pkt {
                                    BrowserToWsta::SelectView { view } => {
                                        selected_view = *view;
                                        let _ = browser_tx.send(WstaToBrowser::SelectedView {
                                            view: selected_view,
                                        });
                                    }

                                    _ => {
                                        if let Some(snively) = pkt.maybe_snively() {
                                            let _ = browser_tx.send(WstaToBrowser::DebugLine {
                                                line: format!("browser -> seek dsta::Snively {:?}", snively),
                                            });

                                            if let Err(e) = seek_cmd_tx.send(SeekBridgeCommand::SendSnively(snively)).await {
                                                let _ = browser_tx.send(WstaToBrowser::Error {
                                                    message: format!("seek bridge command channel closed: {}", e),
                                                });
                                            }
                                        }
                                    }
                                }
                            }

                            ActorMsg::SeekEvent(ev) => {
                                match ev {
                                    SeekBridgeEvent::Connected { remote } => {
                                        let _ = browser_tx.send(WstaToBrowser::Connected {
                                            message: format!("seek connected to {}", remote),
                                        });
                                    }

                                    SeekBridgeEvent::Disconnected { reason } => {
                                        let _ = browser_tx.send(WstaToBrowser::Error {
                                            message: format!("seek disconnected: {}", reason),
                                        });
                                    }

                                    SeekBridgeEvent::DebugLine(line) => {
                                        let _ = browser_tx.send(WstaToBrowser::DebugLine { line });
                                    }

                                    SeekBridgeEvent::MadeBot(value) => {
                                        let _ = browser_tx.send(WstaToBrowser::BackendMadeBot { value });
                                    }

                                    SeekBridgeEvent::GotTtai(value) => {
                                        let _ = browser_tx.send(WstaToBrowser::BackendGotTtai { value });
                                    }

                                    SeekBridgeEvent::GotTick00(value) => {
                                        let _ = browser_tx.send(WstaToBrowser::BackendGotTick00 { value });
                                    }

                                    SeekBridgeEvent::GotTick01(value) => {
                                        let _ = browser_tx.send(WstaToBrowser::BackendGotTick01 { value });
                                    }
                                }
                            }
                        }
                    }

                    else => {
                        break;
                    }
                }
            }
        });
    }

    ActorHandle {
        tx: actor_tx,
        browser_tx,
    }
}
