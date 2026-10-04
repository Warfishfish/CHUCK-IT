# Australian BBQ: notes for Claude Code

Marcus's backyard party game (it used to be called Chuck It / Backyard Brawl). Cartoon blob characters throw things at each other in an Aussie backyard. There are Free for all, Teams and Teddy Heist modes, plus bots, a bar and drunk meter, Dazza the BBQ cook, grab-and-drag, Cheeky mode, and online play with mates.

## Where things are
- `public/index.html` is the whole browser game: Three.js + Web Audio, about 3,100 lines. CSS, HTML and one big script all live in this file.
- `public/room-ws.js` connects the game to the server.
- `server.js` is a tiny Node relay server (rooms + presence). Start it with `npm install` then `npm start` (http://localhost:3000).
- `CHECKLIST.md` is the master list of what's built and what's still to do.
- `RUST_REWRITE_PLAN.md` covers the planned Rust/Bevy rewrite: analysis, tools, risks and a phased checklist.
- `PROMPT.md` and `PROMPT-bot-movement.md` are older hand-off prompts.

## How Marcus wants to work
- Give complete, ready-to-use files, not snippets. Explain things in plain words; Marcus isn't a full-time programmer.
- Before changing the game, read the relevant code. Don't guess from names.
- Every game change:
  1. Test it: solo, plus 2 players online where networking is involved.
  2. Bump the version in `<span class="tag">Backyard Brawl · prototype X.Y.Z</span>`.
  3. Update CHECKLIST.md: move finished things into "Already in the game".
  4. Commit with a clear message and push to GitHub (`main`).
  5. End with this checklist:
     1. In Terminal, press Ctrl+C, then run `npm start`.
     2. Everyone reloads the page. (The trycloudflare link only changes if the `cloudflared` window was restarted.)
- Say plainly what you guessed, simplified or left out.

## Rules already decided
- No trick shots, no wall-bounce bonuses, no voice or speaking sounds. Keep the "+N drunk" pop-up and Dazza chatter about every 30 s.
- Keep the original blob character design in the JavaScript game. The Rust version has four selectable characters (Marcus, 4 Oct 2026): Classic (the original capsule), Pear, Egg and Gumdrop (the default). They differ in looks only, with the same speed and hit sizes for everyone. Models are made by `rust/tools/make_blob.py`; the list is in `rust/crates/bbq_core/src/character.rs`. Dodge is removed.
- Controls: Q swaps items, right-click catches, F grabs/drags/throws someone who's down (otherwise it winds up a throw), R interacts.
- 1v1 with no bots must always work.
- Heist scores only from banking teddies. Stuns don't stack, and there's a 1 s grace after a stun ends.
- Sounds or assets must be Steam-safe (CC0, CC-BY, Pixabay, ZapSplat; no NC/ND), with a licence list.

## Rust rewrite
Follow RUST_REWRITE_PLAN.md one phase at a time, and don't jump ahead.

Decided by Marcus (4 Oct 2026):
1. Desktop first (Mac/Windows/Steam), browser later. The JavaScript game stays as the browser version in the meantime.
2. A player hosts online games, like today. No dedicated server for now.

Put the Rust code in the `rust/` folder (a Cargo workspace) so the browser game keeps working alongside it. `BEHAVIOUR_SPEC.md` lists every tuning number from the JavaScript game; port from that.
