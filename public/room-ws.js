// Connects the game to the Chuck It server (used when the game is not running inside Claude).
(function () {
  if (window.claude && typeof window.claude.use === 'function') return;
  const url = (location.protocol === 'https:' ? 'wss://' : 'ws://') + location.host + '/ws';
  const NAME = /^[a-z0-9][a-z0-9_.-]{0,47}$/;
  const rooms = new Map();
  const connHandlers = new Set();
  let ws = null, connected = false, myPeer = 'me', retry = 1000;

  function sendRaw(o) { if (ws && ws.readyState === 1) ws.send(JSON.stringify(o)); }
  function peerObj(peer, presence) {
    return Object.freeze({ peer, presence: Object.freeze(presence || {}), by: null, isMe: false, sameTab: false, kind: 'viewer', guest: false, updatedAt: Date.now() });
  }
  function connect() {
    ws = new WebSocket(url);
    ws.onopen = () => {
      connected = true; retry = 1000;
      for (const r of rooms.values()) { sendRaw({ t: 'join', room: r.name }); if (Object.keys(r.mine).length) sendRaw({ t: 'pres', room: r.name, presence: r.mine }); }
      connHandlers.forEach(h => h(true));
    };
    ws.onclose = () => {
      const was = connected; connected = false;
      for (const r of rooms.values()) { const left = [...r.peers.values()]; r.peers.clear(); if (left.length) r.fire({ joined: [], updated: [], left }); }
      if (was) connHandlers.forEach(h => h(false));
      setTimeout(connect, retry); retry = Math.min(retry * 2, 10000);
    };
    ws.onmessage = e => {
      let m; try { m = JSON.parse(e.data); } catch { return; }
      if (m.t === 'you') { myPeer = m.peer; return; }
      const r = rooms.get(m.room); if (!r) return;
      if (m.t === 'snap') {
        const joined = [];
        for (const p of m.peers || []) { const pe = peerObj(p.peer, p.presence); r.peers.set(p.peer, pe); joined.push(pe); }
        r.fire({ joined, updated: [], left: [] });
      } else if (m.t === 'up') {
        const had = r.peers.has(m.peer); const pe = peerObj(m.peer, m.presence); r.peers.set(m.peer, pe);
        r.fire(had ? { joined: [], updated: [pe], left: [] } : { joined: [pe], updated: [], left: [] });
      } else if (m.t === 'left') {
        const pe = r.peers.get(m.peer); if (pe) { r.peers.delete(m.peer); r.fire({ joined: [], updated: [], left: [pe] }); }
      }
    };
  }
  function makeRoom(name) {
    const r = { name, mine: {}, peers: new Map(), hs: new Set() };
    r.self = () => ({ peer: myPeer, presence: r.mine, by: null, isMe: true, sameTab: true, kind: 'viewer', guest: false, updatedAt: Date.now() });
    r.fire = ch => { const c = Object.assign({ peers: api.peers() }, ch); r.hs.forEach(h => { try { h(c); } catch (err) { console.error(err); } }); };
    const api = {
      name,
      presence: async patch => { for (const k in patch) { if (patch[k] === null) delete r.mine[k]; else r.mine[k] = patch[k]; } sendRaw({ t: 'pres', room: name, presence: r.mine }); },
      peers: () => [r.self(), ...r.peers.values()],
      onPeers: h => { r.hs.add(h); setTimeout(() => h({ peers: api.peers(), joined: api.peers(), updated: [], left: [] }), 0); return () => r.hs.delete(h); },
      connected: () => connected,
      onConnection: h => { connHandlers.add(h); setTimeout(() => h(connected), 0); return () => connHandlers.delete(h); },
      emit: async () => {}, on: () => () => {},
      leave: async () => { sendRaw({ t: 'leave', room: name }); rooms.delete(name); r.hs.clear(); }
    };
    rooms.set(name, r); r.api = api; sendRaw({ t: 'join', room: name });
    return api;
  }
  const lobby = makeRoom('lobby');
  lobby.join = async name => {
    if (typeof name !== 'string' || !NAME.test(name)) throw { code: 'invalid_argument', message: 'bad room name' };
    return rooms.has(name) ? rooms.get(name).api : makeRoom(name);
  };
  connect();
  window.claude = { use: async n => (n === 'room' ? lobby : null) };
})();
