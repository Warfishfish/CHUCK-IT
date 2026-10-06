// Chuck It game server: serves the game and relays players between browsers.
// Run with:  npm install  then  npm start   (defaults to http://localhost:3000)
const http = require('http');
const fs = require('fs');
const path = require('path');
const { WebSocketServer } = require('ws');

const PORT = process.env.PORT || 3000;
const PUBLIC = path.join(__dirname, 'public');
const ROOM_NAME = /^[a-z0-9][a-z0-9_.-]{0,47}$/;
const MAX_PER_ROOM = 16;
const MAX_PRESENCE_BYTES = 8192;
const MAX_MSG_BYTES = 12288; // one relayed game message, after base64
const TYPES = { '.html': 'text/html; charset=utf-8', '.js': 'text/javascript; charset=utf-8', '.css': 'text/css; charset=utf-8', '.png': 'image/png', '.ico': 'image/x-icon', '.svg': 'image/svg+xml' };

const server = http.createServer((req, res) => {
  let p;
  try { p = decodeURIComponent(new URL(req.url, 'http://x').pathname); } catch { res.writeHead(400); return res.end(); }
  if (p === '/healthz') { res.writeHead(200); return res.end('ok'); }
  if (p === '/') p = '/index.html';
  const file = path.join(PUBLIC, path.normalize(p));
  if (!file.startsWith(PUBLIC)) { res.writeHead(403); return res.end(); }
  fs.readFile(file, (err, data) => {
    if (err) { res.writeHead(404, { 'Content-Type': 'text/plain' }); return res.end('Not found'); }
    res.writeHead(200, { 'Content-Type': TYPES[path.extname(file)] || 'application/octet-stream', 'Cache-Control': 'no-cache' });
    res.end(data);
  });
});

const wss = new WebSocketServer({ server, path: '/ws', maxPayload: 16 * 1024 });
const rooms = new Map(); // room name -> Map(peer -> { ws, presence })
let counter = 0;

function send(ws, msg) { if (ws.readyState === 1) ws.send(typeof msg === 'string' ? msg : JSON.stringify(msg)); }
function broadcast(room, msg, exceptPeer) {
  const r = rooms.get(room); if (!r) return;
  const text = JSON.stringify(msg);
  for (const [peer, v] of r) if (peer !== exceptPeer) send(v.ws, text);
}
function leave(ws, room) {
  const r = rooms.get(room); if (!r || !r.has(ws.peer)) return;
  r.delete(ws.peer); ws.rooms.delete(room);
  if (r.size === 0) rooms.delete(room); else broadcast(room, { t: 'left', room, peer: ws.peer });
}

wss.on('connection', ws => {
  ws.peer = (++counter).toString(36) + Math.random().toString(36).slice(2, 8);
  ws.rooms = new Set(); ws.alive = true; ws.budget = 400;
  ws.on('pong', () => { ws.alive = true; });
  send(ws, { t: 'you', peer: ws.peer });

  ws.on('message', raw => {
    if (ws.budget-- <= 0) return; // flood guard
    let m; try { m = JSON.parse(raw); } catch { return; }
    if (!m || typeof m.room !== 'string' || !ROOM_NAME.test(m.room)) return;
    const room = m.room;
    if (m.t === 'join') {
      let r = rooms.get(room);
      if (!r) { r = new Map(); rooms.set(room, r); }
      if (!r.has(ws.peer) && (r.size >= MAX_PER_ROOM || ws.rooms.size >= 8)) return send(ws, { t: 'err', room, code: 'limit_reached' });
      r.set(ws.peer, { ws, presence: {} }); ws.rooms.add(room);
      send(ws, { t: 'snap', room, peers: [...r].filter(([p]) => p !== ws.peer).map(([peer, v]) => ({ peer, presence: v.presence })) });
      broadcast(room, { t: 'up', room, peer: ws.peer, presence: {} }, ws.peer);
    } else if (m.t === 'pres') {
      const me = rooms.get(room)?.get(ws.peer);
      if (!me || !m.presence || typeof m.presence !== 'object' || Array.isArray(m.presence)) return;
      if (Buffer.byteLength(JSON.stringify(m.presence)) > MAX_PRESENCE_BYTES) return;
      me.presence = m.presence;
      broadcast(room, { t: 'up', room, peer: ws.peer, presence: m.presence }, ws.peer);
    } else if (m.t === 'msg') {
      // The Rust version's game messages: relayed as they are (base64 text) to one peer (`to`)
      // or to everybody else in the room. The server never looks inside.
      const r = rooms.get(room);
      if (!r || !r.has(ws.peer) || typeof m.d !== 'string' || m.d.length > MAX_MSG_BYTES) return;
      const out = JSON.stringify({ t: 'msg', room, from: ws.peer, d: m.d });
      if (typeof m.to === 'string') { const dest = r.get(m.to); if (dest) send(dest.ws, out); }
      else for (const [peer, v] of r) if (peer !== ws.peer) send(v.ws, out);
    } else if (m.t === 'leave') {
      leave(ws, room);
    }
  });
  ws.on('close', () => { for (const room of [...ws.rooms]) leave(ws, room); });
});

setInterval(() => { for (const ws of wss.clients) ws.budget = 400; }, 1000);
setInterval(() => {
  for (const ws of wss.clients) { if (!ws.alive) { ws.terminate(); continue; } ws.alive = false; ws.ping(); }
}, 20000);

server.listen(PORT, () => console.log(`Chuck It is running on http://localhost:${PORT}`));
