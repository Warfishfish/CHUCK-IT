// Verification only: observe outgoing appearance packets without modifying them.
const path = require('path');
const fs = require('fs');
const WS = require(path.resolve(__dirname, '../../node_modules/ws'));
const original = WS.prototype.send;
WS.prototype.send = function(data, ...args) {
  try {
    const m = JSON.parse(data.toString());
    if (m.t === 'msg' && [1, 2, 4, 10].includes(Buffer.from(m.d, 'base64')[0])) {
      fs.appendFileSync(path.join(__dirname, 'network/appearance-packets.jsonl'), JSON.stringify(m) + '\n');
    }
  } catch (_) {}
  return original.call(this, data, ...args);
};
