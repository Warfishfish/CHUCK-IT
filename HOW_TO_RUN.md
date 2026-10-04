# How to open the game from Terminal

## Rust desktop version
```
cd ~/Claude/australian-bbq/rust && export PATH=$HOME/.cargo/bin:$PATH && cargo run -p bbq_app --release
```
Start straight into a round: add `-- --mode heist` (or `teams`, `ffa`) at the end.
Keys: Enter starts a round, F12 changes mode, - / = change bots, F10 bots on/off, F11 difficulty.

## Browser (JavaScript) version
```
cd ~/Claude/australian-bbq && npm start
```
Then open http://localhost:3000
