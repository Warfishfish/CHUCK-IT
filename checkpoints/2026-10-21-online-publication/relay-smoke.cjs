const WebSocket = require('ws');
const room = 'verify-' + Date.now();
const url = process.argv[2];
const peers = [new WebSocket(url), new WebSocket(url)];
let joined=0,received=0, ids=[], sent=false;
const timer=setTimeout(()=>finish(new Error('Relay verification timed out')),20000);
let done=false;
function finish(error) { if(done)return;done=true;clearTimeout(timer);for(const ws of peers)ws.close(); if(error){console.error(error.message);process.exitCode=1;} else console.log('PASS: two peers joined through Cloudflare; messages relayed in both directions. Test room closed.'); }
peers.forEach((ws,i)=>{
 ws.on('error',finish);
 ws.on('open',()=>ws.send(JSON.stringify({t:'join',room})));
 ws.on('message',raw=>{
  const m=JSON.parse(raw);
  if(m.t==='you')ids[i]=m.peer;
  if(m.t==='snap')joined++;
  if(joined===2&&!sent){sent=true;peers.forEach((p,j)=>p.send(JSON.stringify({t:'msg',room,d:Buffer.from('relay-check-'+j).toString('base64')})));}
  if(m.t==='msg'){
   if(Buffer.from(m.d,'base64').toString()!=='relay-check-'+(1-i))return finish(new Error('Unexpected relayed payload'));
   received++;if(received===2)finish();
  }
 });
});
