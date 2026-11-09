# Browser build — 9 November 2026

**Browser bundle source:** `ac513c9` (`Add bot team controls, ute traversal, and carry camera`)
**Network protocol:** 28 (unchanged; no wire-format change)
**Live play link:** https://twins-hepatitis-led-terrorism.trycloudflare.com/rust/

## Published build

Rebuilt with `rust/tools/build_web.sh` into `public/rust/`. The bundle contains 81 files and is about 111 MB. Release metadata points to source commit `ac513c9`.

## Verification

- The game page loaded to the main menu in the browser.
- The page, JavaScript, wasm, compressed wasm, release metadata and Smoko Servo model returned HTTP 200.
- Public SHA-256 hashes matched local files:
  - `bbq.js`: `cc1455423492b8463b49678ca30bbfa72be4c73dd6073c1e31fc3ff9d170c8de`
  - `bbq_bg.wasm`: `768b7a6ef403a1675682950697c0c587fe5b60c65a9c555b0f9652f3fb8097e1`
  - `bbq_bg.wasm.gz`: `21570ce897618a3c1b2b2e7d5a728a8b2ae4075c3266a4a970373ac7e76bb674`
  - `release.json`: `51463c5405dcfbb840f23d5d86b6fdb60d4765cbb6aa7c16cd590f751620adb1`
  - `assets/models/smoko_servo.glb`: `71d8c92730d3421629f4b739cc398b9bf8a4380919642fba14c72b8dbfc517ab`
- Two browser peers joined the public relay and exchanged messages in both directions.

## Rollback and availability

The previous bundle is backed up at `/Users/marcus/Claude/australian-bbq-browser-backup-ac513c9/rust`. Restore that directory as `public/rust` to roll back this browser publication.

The link is a temporary Cloudflare Quick Tunnel. It works only while the Mac, the server on port 3000 and the tunnel process remain running; a replacement tunnel will have a different URL. Refresh the browser and create a fresh yard after this update. Old protocol-28 clients can still join because the protocol did not change.
