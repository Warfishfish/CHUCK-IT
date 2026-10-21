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

## Temporary Cloudflare link for friends

Rebuild the browser version after Rust changes: `./rust/tools/build_web.sh` from the repository root.
Keep the game server running in one Terminal:

```sh
cd /Users/marcus/Claude/australian-bbq
npm start
```

In another Terminal, create a temporary tunnel (cloudflared is already installed on this Mac):

```sh
cloudflared tunnel --config /dev/null --no-autoupdate --url http://localhost:3000
```

Share the printed `https://….trycloudflare.com` address **with `/rust/` on the end**.
Everyone opens that same link; choose **With mates → Host a yard**, then share the room code.
Friends choose **Join their yard**. Use a WebGPU-capable desktop browser. Refresh old tabs after updates.
The Mac must stay awake and both processes must keep running. A restarted quick tunnel gives a new address.

GitHub stores the source. Generated `public/rust/` files stay local and are rebuilt separately;
a Git push alone does not update this browser build or create a live tunnel.
