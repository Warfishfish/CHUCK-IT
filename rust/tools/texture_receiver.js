// Tiny local receiver for the comparison tools (see compare.md): the browser game POSTs PNG data here and it is saved to a folder.
// Run:  node rust/tools/texture_receiver.js some/folder
const http = require('http'), fs = require('fs'), path = require('path');
const dir = process.argv[2];
http.createServer((req, res) => {
  res.setHeader('Access-Control-Allow-Origin', '*');
  res.setHeader('Access-Control-Allow-Headers', '*');
  if (req.method === 'OPTIONS') { res.writeHead(204); return res.end(); }
  const name = new URL(req.url, 'http://x').searchParams.get('name').replace(/[^a-z0-9_.-]/gi, '');
  let body = '';
  req.on('data', d => body += d);
  req.on('end', () => {
    const b64 = body.replace(/^data:image\/png;base64,/, '');
    fs.writeFileSync(path.join(dir, name), Buffer.from(b64, 'base64'));
    res.writeHead(200); res.end('ok');
  });
}).listen(3999, () => console.log('receiver on 3999'));
