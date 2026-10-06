// Checks the relay in server.js: two peers join a room, a message goes to one peer or to
// everybody else, and strangers outside the room get nothing. Run from the repo folder:
//   node rust/tools/relay_test.js
const { spawn } = require('child_process');
const path = require('path');
const WebSocket = require(path.join(__dirname, '..', '..', 'node_modules', 'ws'));
const PORT = 3190 + Math.floor(Math.random() * 50);
const srv = spawn('node', [path.join(__dirname, '..', '..', 'server.js')], { env: { ...process.env, PORT } });
const fail = m => { console.error('FAIL', m); srv.kill(); process.exit(1); };
setTimeout(() => fail('timeout'), 8000);
function client(room) {
  return new Promise(res => {
    const ws = new WebSocket(`ws://localhost:${PORT}/ws`); const got = [];
    ws.on('message', d => { const m = JSON.parse(d); got.push(m); if (m.t === 'you') { ws.peer = m.peer; if (room) ws.send(JSON.stringify({ t: 'join', room })); res(Object.assign(ws, { got })); } });
  });
}
const wait = ms => new Promise(r => setTimeout(r, ms));
setTimeout(async () => {
  const a = await client('rbq-test'), b = await client('rbq-test'), c = await client('rbq-other');
  await wait(200);
  a.send(JSON.stringify({ t: 'msg', room: 'rbq-test', d: 'aGk=' }));
  b.send(JSON.stringify({ t: 'msg', room: 'rbq-test', to: a.peer, d: 'eW8=' }));
  c.send(JSON.stringify({ t: 'msg', room: 'rbq-test', d: 'bm8=' })); // not in the room: dropped
  a.send(JSON.stringify({ t: 'msg', room: 'rbq-test', d: 'x'.repeat(20000) })); // too big: dropped
  await wait(300);
  const msgs = x => x.got.filter(m => m.t === 'msg');
  if (msgs(b).length !== 1 || msgs(b)[0].d !== 'aGk=' || msgs(b)[0].from !== a.peer) fail('b should get a\'s broadcast: ' + JSON.stringify(msgs(b)));
  if (msgs(a).length !== 1 || msgs(a)[0].d !== 'eW8=') fail('a should get b\'s direct message: ' + JSON.stringify(msgs(a)));
  if (msgs(c).length !== 0) fail('c is in another room');
  console.log('relay ok');
  srv.kill(); process.exit(0);
}, 800);
