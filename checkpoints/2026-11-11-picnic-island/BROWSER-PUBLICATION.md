# Smoko picnic island browser build · 12 November 2026

Source commit: `5dbdd56` · protocol 28 · browser bundle: `public/rust/`

## Build and checks

- `rust/tools/build_web.sh` completed successfully. The 81-file browser bundle is 112 MB.
- The local `server.js` served the game page, release metadata, both updated GLB models and the updated picnic shade texture with HTTP 200 responses.
- The three shipped picnic assets byte-match their Rust source copies. The JavaScript loader is unchanged; the WebAssembly binary and compressed copy were rebuilt.
- `cargo test --workspace` passed for source commit `5dbdd56`; the earlier restricted run needed a retry with local network access for the temporary relay integration test.
- The existing Cloudflare hostname did not resolve during this session, and no `cloudflared` process was running. This verifies the local browser bundle, not a live public tunnel.

## Bundle hashes

```text
bbq.js cc1455423492b8463b49678ca30bbfa72be4c73dd6073c1e31fc3ff9d170c8de
bbq_bg.wasm 91594936c4db3deea2e226da803286468287ac8386a735ed2b104f67b918fd92
bbq_bg.wasm.gz 25d657cc66af46857454522be35c9228f85f145e552237f23be96f8fff04770c
release.json 5f647c5c1d7ad6d70f89b22103802374ccd0bb455e0ae0669238dca80f3c12e5
smoko_props.glb 49257e9cf82044b7dc72b9e4d012b8f98970009a07789da956dfdd4edc709580
smoko_surroundings.glb 0c826419c31ca34c17bf34a0736092669b571f6a33e714adbfd18314d2ea996c
smoko_picnic_shade.png 2e14ea4912a017be91dee38e60ccb67cb7a0af7f1712b26debc67e8bcba3db43
```
