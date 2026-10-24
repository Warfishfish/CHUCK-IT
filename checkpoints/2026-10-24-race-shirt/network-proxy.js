const WS=require('/Users/marcus/Claude/australian-bbq/node_modules/ws');
const server=new WS.WebSocketServer({port:3001});
server.on('connection',client=>{
 const upstream=new WS('ws://localhost:3000/ws');
 upstream.on('message',data=>{if(client.readyState===WS.OPEN)client.send(data.toString());});
 client.on('message',data=>{
  const forward=()=>{if(upstream.readyState===WS.OPEN)upstream.send(data.toString());};
  try {const m=JSON.parse(data);const bytes=Buffer.from(m.d||'','base64');
   if(m.t==='msg'&&bytes.length===3&&bytes[0]===8&&bytes[1]===6&&bytes[2]===0){console.log('Delaying carry release by 350ms to exercise host-measured throw threshold');setTimeout(forward,350);return;}
  }catch{}
  forward();
 });
 client.on('close',()=>upstream.close());upstream.on('close',()=>client.close());
});
console.log('Race regression proxy listening on 3001');
