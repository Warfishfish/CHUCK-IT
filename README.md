# Chuck It: your own game server

This folder runs Chuck It on your own web address. Your mates just open the link, with no accounts needed.

## Option A: host from your Mac tonight (about 5 minutes)

1. Install Node.js (the "LTS" version) from https://nodejs.org if you don't have it. Check with `node -v` in Terminal.
2. In Terminal, go into this folder and start the server:
   ```
   cd ~/Downloads/chuck-it-server
   npm install
   npm start
   ```
   You should see: `Chuck It is running on http://localhost:3000`
3. Open http://localhost:3000 on your Mac to check it works.
4. To give mates an https link, open a **second** Terminal window and run:
   ```
   brew install cloudflared        # one time only (needs Homebrew: https://brew.sh)
   cloudflared tunnel --url http://localhost:3000
   ```
   It prints a link like `https://random-words.trycloudflare.com`. Send that to your mates.

Notes: the link changes each time you run the tunnel, and it only works while your Mac is awake with both Terminal windows open.

## Option B: a permanent https address (free, about 15 minutes)

Uses Render (https://render.com), which gives you a fixed link like `https://chuck-it.onrender.com`.

1. Make a free GitHub account, create a new repository called `chuck-it`, and upload everything in this folder **except** `node_modules`.
2. Make a free Render account and sign in with GitHub.
3. New → Web Service → pick the `chuck-it` repository, then set:
   - Runtime: **Node**
   - Build command: `npm install`
   - Start command: `npm start`
   - Instance type: **Free**
4. Click Create. After a minute or two Render shows your https link.

On the free plan, the server goes to sleep after 15 minutes without players. The first visit after that takes up to a minute to wake it.

## Playing

- Everyone opens the link. The host clicks **Host a yard** and shares the code (e.g. YARD-KBZQ), or mates click the yard in the "open yards" list.
- The host's browser runs the yard, so the host should keep that tab open and in front.

## Files

- `server.js`: serves the game and passes messages between players.
- `public/index.html`: the game.
- `public/room-ws.js`: connects the game to the server.
