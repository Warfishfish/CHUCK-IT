# How to open the game from Terminal

## Rust desktop version
```
cd ~/Claude/australian-bbq/rust && export PATH=$HOME/.cargo/bin:$PATH && cargo run -p bbq_app --release
```
Start straight into a round: add `-- --mode heist` (or `teams`, `ffa`) at the end.
Keys: Enter starts a round, F12 changes mode, - / = change bots, F10 bots on/off, F11 difficulty.

## Rust version in the browser (with online play)
Build it (about 10 minutes the first time, a few minutes after that). Do this again after every
change to the Rust game:
```
cd ~/Claude/australian-bbq/rust && ./tools/build_web.sh
```
Then start the server (the same one as the old browser game):
```
cd ~/Claude/australian-bbq && npm start
```
Open http://localhost:3000/rust/ (needs WebGPU: a recent Chrome, Edge, Safari or Firefox).

Playing online: one person clicks **With mates → Host a yard** and reads out the room code. The
others open the same page (on the trycloudflare link if they are not on your Wi-Fi, with `/rust/`
on the end), click **With mates**, type the code and **Join their yard**. The server link fills
itself in. Desktop players can join the same yard if their build is up to date.

The first visit downloads about 19 MB (14 MB game + 5 MB pictures and models).

## Browser (JavaScript) version
```
cd ~/Claude/australian-bbq && npm start
```
Then open http://localhost:3000
