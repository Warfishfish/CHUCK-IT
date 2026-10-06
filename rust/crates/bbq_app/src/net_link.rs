//! The connection to the relay server (`server.js`, the same one the browser game uses): a
//! WebSocket kept on its own thread, talking to the game through two channels so the game never
//! waits on the network. The relay only passes bytes along (inside `msg` packets, as base64
//! text); what they mean is decided by `bbq_core::net`.

use std::io::ErrorKind;
use std::sync::Mutex;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::Duration;

use base64::Engine;
use base64::engine::general_purpose::STANDARD as B64;
use serde_json::{Value, json};
use tungstenite::stream::MaybeTlsStream;
use tungstenite::{Message, WebSocket};

/// What the game asks the connection to do.
#[derive(Debug)]
pub enum Out {
    Join(String),
    /// Send bytes to one peer (`to`) or to everybody else in the room.
    Msg { room: String, to: Option<String>, data: Vec<u8> },
    Leave(String),
    Close,
}

/// What the connection tells the game.
#[derive(Debug, PartialEq, Eq)]
pub enum In {
    /// Connected; this is our peer name on the relay.
    Open { peer: String },
    /// Joined a room: these peers are already in it.
    Snap { room: String, peers: Vec<String> },
    Up { room: String, peer: String },
    Left { room: String, peer: String },
    Msg { room: String, from: String, data: Vec<u8> },
    /// The relay said no (for example the room is full).
    Refused { room: String, code: String },
    /// The connection ended or never started.
    Closed(String),
}

pub struct Link {
    pub out: Sender<Out>,
    // behind a lock so the link can live in a Bevy resource (a bare Receiver is not Sync)
    inp: Mutex<Receiver<In>>,
}

impl Link {
    /// Start connecting to `url` (see `ws_url`) on a new thread. Returns at once.
    pub fn connect(url: String) -> Link {
        let (out, out_rx) = channel();
        let (in_tx, inp) = channel();
        std::thread::spawn(move || {
            let reason = run(&url, &out_rx, &in_tx);
            let _ = in_tx.send(In::Closed(reason));
        });
        Link { out, inp: Mutex::new(inp) }
    }

    /// The next thing the connection has to tell us, if anything.
    pub fn poll(&self) -> Option<In> {
        self.inp.lock().ok()?.try_recv().ok()
    }

    pub fn recv_timeout(&self, t: Duration) -> Option<In> {
        self.inp.lock().ok()?.recv_timeout(t).ok()
    }

    pub fn send(&self, room: &str, to: Option<&str>, data: Vec<u8>) {
        let _ = self.out.send(Out::Msg { room: room.to_string(), to: to.map(str::to_string), data });
    }
}

impl Drop for Link {
    fn drop(&mut self) {
        let _ = self.out.send(Out::Close);
    }
}

/// What people type for the server turns into the address of the WebSocket:
/// `localhost:3000` -> `ws://localhost:3000/ws`, `https://x.trycloudflare.com` ->
/// `wss://x.trycloudflare.com/ws`. Anything with spaces or no host is refused.
pub fn ws_url(input: &str) -> Option<String> {
    let s = input.trim();
    if s.is_empty() || s.chars().any(|c| c.is_whitespace() || c.is_control()) {
        return None;
    }
    let (scheme, rest) = if let Some(r) = s.strip_prefix("https://").or_else(|| s.strip_prefix("wss://")) {
        ("wss", r)
    } else if let Some(r) = s.strip_prefix("http://").or_else(|| s.strip_prefix("ws://")) {
        ("ws", r)
    } else if s.contains("://") {
        return None;
    } else {
        ("ws", s)
    };
    let host = rest.trim_end_matches('/').split('/').next().unwrap_or("");
    let bare = host.rsplit_once(':').map_or(host, |(h, _)| h);
    if bare.is_empty() || !bare.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '.') {
        return None;
    }
    Some(format!("{scheme}://{host}/ws"))
}

fn set_timeout(ws: &mut WebSocket<MaybeTlsStream<std::net::TcpStream>>) {
    let t = Some(Duration::from_millis(40));
    match ws.get_mut() {
        MaybeTlsStream::Plain(s) => {
            let _ = s.set_read_timeout(t);
        }
        MaybeTlsStream::NativeTls(s) => {
            let _ = s.get_mut().set_read_timeout(t);
        }
        _ => {}
    }
}

fn would_block(e: &tungstenite::Error) -> bool {
    matches!(e, tungstenite::Error::Io(io) if matches!(io.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut))
}

/// The thread's whole life. Returns why it ended.
fn run(url: &str, out_rx: &Receiver<Out>, in_tx: &Sender<In>) -> String {
    let (mut ws, _) = match tungstenite::connect(url) {
        Ok(c) => c,
        Err(e) => return format!("could not connect: {e}"),
    };
    set_timeout(&mut ws);
    loop {
        // everything the game wants sent
        while let Ok(o) = out_rx.try_recv() {
            let text = match o {
                Out::Join(room) => json!({ "t": "join", "room": room }),
                Out::Leave(room) => json!({ "t": "leave", "room": room }),
                Out::Msg { room, to, data } => {
                    let mut m = json!({ "t": "msg", "room": room, "d": B64.encode(data) });
                    if let Some(to) = to {
                        m["to"] = Value::String(to);
                    }
                    m
                }
                Out::Close => {
                    let _ = ws.close(None);
                    return "closed".to_string();
                }
            };
            if let Err(e) = ws.send(Message::text(text.to_string())) {
                return format!("send failed: {e}");
            }
        }
        // everything that has arrived
        match ws.read() {
            Ok(Message::Text(t)) => {
                if let Some(ev) = parse(&t) {
                    if in_tx.send(ev).is_err() {
                        return "game closed".to_string();
                    }
                }
            }
            Ok(Message::Close(_)) => return "the server closed the connection".to_string(),
            Ok(_) => {}
            Err(e) if would_block(&e) => {}
            Err(e) => return format!("connection lost: {e}"),
        }
    }
}

/// One text packet from the relay, as an event (anything odd is ignored).
pub fn parse(text: &str) -> Option<In> {
    let v: Value = serde_json::from_str(text).ok()?;
    let s = |k: &str| v.get(k).and_then(Value::as_str).map(str::to_string);
    Some(match v.get("t")?.as_str()? {
        "you" => In::Open { peer: s("peer")? },
        "snap" => In::Snap {
            room: s("room")?,
            peers: v.get("peers")?.as_array()?.iter().filter_map(|p| p.get("peer")?.as_str().map(str::to_string)).collect(),
        },
        "up" => In::Up { room: s("room")?, peer: s("peer")? },
        "left" => In::Left { room: s("room")?, peer: s("peer")? },
        "err" => In::Refused { room: s("room")?, code: s("code")? },
        "msg" => In::Msg { room: s("room")?, from: s("from")?, data: B64.decode(s("d")?).ok()? },
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::{Child, Command, Stdio};
    use std::time::Instant;

    #[test]
    fn what_people_type_becomes_a_websocket_address() {
        assert_eq!(ws_url("localhost:3000").as_deref(), Some("ws://localhost:3000/ws"));
        assert_eq!(ws_url("  192.168.1.5:3000/ ").as_deref(), Some("ws://192.168.1.5:3000/ws"));
        assert_eq!(ws_url("http://localhost:3000/").as_deref(), Some("ws://localhost:3000/ws"));
        assert_eq!(ws_url("https://cool-name.trycloudflare.com").as_deref(), Some("wss://cool-name.trycloudflare.com/ws"));
        assert_eq!(ws_url("https://cool-name.trycloudflare.com/some/page").as_deref(), Some("wss://cool-name.trycloudflare.com/ws"));
        assert_eq!(ws_url(""), None);
        assert_eq!(ws_url("two words"), None);
        assert_eq!(ws_url("ftp://x"), None);
        assert_eq!(ws_url("https://"), None);
        assert_eq!(ws_url("bad_host!"), None);
    }

    #[test]
    fn relay_packets_become_events() {
        assert_eq!(parse(r#"{"t":"you","peer":"a1"}"#), Some(In::Open { peer: "a1".into() }));
        assert_eq!(
            parse(r#"{"t":"snap","room":"r","peers":[{"peer":"x","presence":{}},{"peer":"y","presence":{}}]}"#),
            Some(In::Snap { room: "r".into(), peers: vec!["x".into(), "y".into()] })
        );
        assert_eq!(parse(r#"{"t":"up","room":"r","peer":"x","presence":{}}"#), Some(In::Up { room: "r".into(), peer: "x".into() }));
        assert_eq!(parse(r#"{"t":"left","room":"r","peer":"x"}"#), Some(In::Left { room: "r".into(), peer: "x".into() }));
        assert_eq!(parse(r#"{"t":"err","room":"r","code":"limit_reached"}"#), Some(In::Refused { room: "r".into(), code: "limit_reached".into() }));
        assert_eq!(parse(r#"{"t":"msg","room":"r","from":"x","d":"aGk="}"#), Some(In::Msg { room: "r".into(), from: "x".into(), data: b"hi".to_vec() }));
        // rubbish is ignored, not a crash
        for bad in ["", "nope", "[]", r#"{"t":5}"#, r#"{"t":"msg","room":"r","from":"x","d":"%%%"}"#, r#"{"t":"unknown"}"#] {
            assert_eq!(parse(bad), None, "{bad}");
        }
    }

    /// Starts the real relay (`node server.js`) on a spare port; None if node is not installed.
    fn start_server(port: u16) -> Option<Child> {
        let repo = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../server.js");
        if !repo.exists() {
            return None;
        }
        let child = Command::new("node")
            .arg(repo)
            .env("PORT", port.to_string())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .ok()?;
        std::thread::sleep(Duration::from_millis(700));
        Some(child)
    }

    fn wait_for<T>(link: &Link, what: &str, mut pick: impl FnMut(In) -> Option<T>) -> T {
        let end = Instant::now() + Duration::from_secs(5);
        while Instant::now() < end {
            if let Some(ev) = link.recv_timeout(Duration::from_millis(100)) {
                if let Some(v) = pick(ev) {
                    return v;
                }
            }
        }
        panic!("timed out waiting for {what}");
    }

    #[test]
    fn two_links_talk_through_the_real_relay() {
        let port = 3400 + (std::process::id() % 400) as u16;
        let Some(mut server) = start_server(port) else {
            eprintln!("node not found: skipping the live relay test");
            return;
        };
        let url = ws_url(&format!("localhost:{port}")).unwrap();
        let a = Link::connect(url.clone());
        let b = Link::connect(url);
        let a_peer = wait_for(&a, "a to connect", |e| if let In::Open { peer } = e { Some(peer) } else { None });
        let b_peer = wait_for(&b, "b to connect", |e| if let In::Open { peer } = e { Some(peer) } else { None });
        a.out.send(Out::Join("rbq-test".into())).unwrap();
        wait_for(&a, "a's snap", |e| matches!(e, In::Snap { .. }).then_some(()));
        b.out.send(Out::Join("rbq-test".into())).unwrap();
        let peers = wait_for(&b, "b's snap", |e| if let In::Snap { peers, .. } = e { Some(peers) } else { None });
        assert_eq!(peers, vec![a_peer.clone()]);
        wait_for(&a, "a told b came", |e| matches!(e, In::Up { peer, .. } if peer == b_peer).then_some(()));
        // a message to everybody else, and one straight back to a
        a.send("rbq-test", None, vec![1, 2, 3]);
        let got = wait_for(&b, "b's message", |e| if let In::Msg { from, data, .. } = e { Some((from, data)) } else { None });
        assert_eq!(got, (a_peer.clone(), vec![1, 2, 3]));
        b.send("rbq-test", Some(&a_peer), vec![9]);
        let got = wait_for(&a, "a's message", |e| if let In::Msg { data, .. } = e { Some(data) } else { None });
        assert_eq!(got, vec![9]);
        // b leaves: a is told
        drop(b);
        wait_for(&a, "a told b left", |e| matches!(e, In::Left { peer, .. } if peer == b_peer).then_some(()));
        let _ = server.kill();
    }

    #[test]
    fn a_dead_address_reports_closed_not_a_hang() {
        let l = Link::connect("ws://127.0.0.1:1/ws".to_string());
        let why = wait_for(&l, "closed", |e| if let In::Closed(w) = e { Some(w) } else { None });
        assert!(why.contains("could not connect"), "{why}");
    }
}
