# Smoko Arena circular road browser publication · 12 November 2026

Source commit: `44eb0c0` · protocol 28 · browser bundle: `public/rust/`

## Build and checks

- `rust/tools/build_web.sh` completed successfully. The generated browser bundle is 111 MB.
- Local `npm start` served `/rust/`, `/rust/release.json` and `/rust/bbq_bg.wasm` with HTTP 200 responses.
- WebAssembly and its gzip copy were rebuilt. The JavaScript loader is unchanged.
- `cargo test --workspace` passed on the source: 263 `bbq_app` tests and 452 `bbq_core` tests, with one ignored app test.
- The bundle was verified locally. No Cloudflare tunnel was running, so this publication does not claim a live public URL was updated.

## Bundle hashes

```text
bbq.js cc1455423492b8463b49678ca30bbfa72be4c73dd6073c1e31fc3ff9d170c8de
bbq_bg.wasm 9b08ab7d9830a3082df47589ff542c2b9424f4db3c18ff2345b96eedfba20448
bbq_bg.wasm.gz 11aef319a2e544fa1c63470c5aebbfd07aeed6ea01dc3b094a60d92204615e67
release.json 6488d7fe810a159e6fa76a88374d5bf8c90bd97ba58d2726d0b2b69c21b09e92
```
