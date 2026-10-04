# Comparing the Rust game with the browser game

Both games can save an 800 x 600 picture from the same camera spot, and `pngtool.py` shows how
far apart they are. Nothing here is part of the game; it is only for checking the look.

## 1. The browser side

1. Start this repo's game server on its own port (not your own `npm start` one): in Claude Code
   use `preview_start` with the `bbq-browser-game` entry in `.claude/launch.json`, or run
   `PORT=3100 node server.js`.
2. `python3 rust/tools/make_ref.py` makes `public/_ref.html` (git-ignored): the same game with
   one extra line, `window.__ev`, that runs code inside the game's own scope.
3. Open `http://localhost:3100/_ref.html`, bring that tab to the front (the game does not draw
   in a hidden tab) and run the JavaScript in `browser_gallery.js`. It pauses the game, lines the
   items up against the sky at the same spots the Rust gallery uses, and saves the canvas as a
   PNG through a tiny receiver (`texture_receiver.js`).

## 2. The Rust side

```sh
rust/tools/rust_shot.sh out.png --gallery                 # the line-up of items
rust/tools/rust_shot.sh out.png --calib                   # one white ball, for the lighting
rust/tools/rust_shot.sh out.png --cam x,y,z,yaw,pitch,fov # any view (degrees; yaw 0 looks along -z)
```
It opens a small window for a moment, saves the picture and quits.

## 3. Compare

```sh
python3 rust/tools/pngtool.py diff browser.png rust.png diff.png    # average difference + a picture of it
python3 rust/tools/pngtool.py side browser.png rust.png side.png    # left browser, right Rust
python3 rust/tools/pngtool.py region browser.png rust.png x0,y0,x1,y1
python3 rust/tools/fit_light.py ball.png                            # lighting numbers from the white ball
```

## How the colours match

three.js r149 does its colour maths on the raw hex numbers and shows them with no sRGB step.
Bevy works in linear light and encodes to sRGB at the end. `bbq_app/src/lighting.rs` makes Bevy
behave like three: hex colours go in as linear numbers, the camera is HDR with no tone mapping, and
`assets/shaders/display_raw.wgsl` undoes the final sRGB step. The three lights are scaled with
the numbers from the white-ball test, which the browser's own picture fits to 3 decimal places
(ambient 0.376/0.433/0.369, up 0.182/0.164/0.250, sun 0.880/0.827/0.723).

Textures are exported from the browser game as PNG files (every drawn canvas is logged by
`make_ref.py`). The ones in use are in `bbq_app/assets/textures/`.
