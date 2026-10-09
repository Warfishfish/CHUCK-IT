#!/bin/zsh
# Builds the Rust game for the browser into ../public/rust/, where `npm start` (server.js) serves
# it at http://localhost:3000/rust/ next to the old browser game. Needs, once:
#   rustup target add wasm32-unknown-unknown
#   cargo install wasm-bindgen-cli --version 0.2.129 --locked
set -e
cd "$(dirname "$0")/.."
export PATH=$HOME/.cargo/bin:$PATH
OUT=../public/rust
cargo build -p bbq_app --profile web --target wasm32-unknown-unknown
rm -rf $OUT
mkdir -p $OUT
wasm-bindgen --target web --no-typescript --out-dir $OUT --out-name bbq \
  target/wasm32-unknown-unknown/web/bbq_app.wasm
cp web/index.html $OUT/
# a squeezed copy of the big file: the server sends it to browsers that accept it (most do)
gzip -k -9 $OUT/bbq_bg.wasm
cp -R crates/bbq_app/assets $OUT/assets
echo "Built: $(du -sh $OUT | cut -f1) in public/rust. Run npm start, then open http://localhost:3000/rust/"
