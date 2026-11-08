# Smoko vividness browser publication · 8 November 2026

Source commit `b7faf26e25ec26ece4af2e583498cb8efde35b7f`, protocol 28. Live play: https://honest-motion-cleaners-holdem.trycloudflare.com/rust/ .

The current public browser bundle was backed up before replacement at `/Users/marcus/Claude/australian-bbq-browser-before-concept-vivid-2-20261108`. The update changes the compiled WebAssembly and release metadata; the JavaScript wrapper, game protocol (28), and existing game assets are unchanged. Asset tree has 75 files and matches the tested private build.

Verified public hashes for JS, compressed and decompressed WASM, HTML, pointer-lock helper, release metadata, and the magpie model. Two public browser peers exchanged messages in both directions. Chrome loaded the public map-selection preview at 1100×713 with no failed HTTP responses or page errors. Screenshot and hashes are in `evidence/public-browser-map.png` and `evidence/publication-verification.json`.

Refresh both host and guests before creating a new room. The Quick Tunnel URL is temporary and requires this Mac, the local server on port 3000, and `cloudflared` to stay running. Roll back the browser bundle as a unit by restoring `public/rust` from the backup above, then reload clients.
