# Cartoon refinement checkpoint

14 October 2026 — implemented and verified, awaiting Marcus’s review. Starting commit dc75e0bb7089d2358954022aa642dc536921a829, main. Active repo /Users/marcus/Claude/australian-bbq. Starting work is the complete uncommitted lighting and character milestones; hashes and status saved here. No unrelated changes found.

User now requests: (1) slightly more cartoon-like presentation, preserving clearer faces; review it before (2) dildo and pool-noodle physics; then return to hit feedback, backyard and lobby. No commit or push until review. Old plans are historical context.

Plan: modest colour/sky/fill-light tuning, slightly bolder correct outlines, softer toy-like skin sheen and less cloth micro-detail. Preserve meshes, expressions, scene geometry, collision and gameplay. Reuse existing assets; no paid generation. Capture matching portraits, backyard and 11-bot views; run relevant checks and compare timings.

Launch: cd /Users/marcus/Claude/australian-bbq/rust && ~/.cargo/bin/cargo run -p bbq_app --release

Assets: rust/crates/bbq_app/assets. Reference images: ChatGPT project mirror graphics-concepts and implemented-screenshots/pop.png. Previous evidence: ../2026-10-14-graphics-m2/evidence and ../2026-10-13-graphics-m1/evidence.

Recovery: starting-work.tar.gz contains every modified/new implementation file at the start (including textures); STARTING-FILES.sha256 verifies them. Extract ONLY into a separate clean checkout at the starting commit, never over later work. Existing M1/M2 patch recovery also remains available. /tmp/bbq-cartoon-baseline is a temporary starting release binary. Prior verification: 177 app tests passed, 1 ignored; M2 release build passed; 11-bot mean 7.845 ms.

## Completed result — awaiting review

The cartoon refinement is implemented and verified. Physics has not been started: the user requested review between milestones.

### Exact changes

- `lighting.rs`: Pop fill light 0.48 → 0.56; saturation 1.05 → 1.12; warmth 0.025 → 0.03; contrast 1.035 → 1.045; vignette 0.18 → 0.14. Sun intensity is preserved. Sky gradient changes from grey-blue to clearer blue.
- `style.rs`: modest palette boost (fade 0 → −0.04, lawn saturation 0.84 → 0.96). Skin roughness 0.68 → 0.60 and clearcoat 0.08 → 0.12. Pop cloth keeps colour/dirt but drops its fine normal map, giving simpler shading. Current/Clay keep their normal map.
- `models.rs`: Pop lawn tint shifts from slightly reddish/olive to greener, using the same grass textures.
- `characters.rs`: correct outlines become slightly bolder: head/torso 0.013 → 0.016; hands/feet 0.011 → 0.013; clothing 0.009 → 0.010. The earlier fix excluding eyes and the Body container remains.
- New `rust/tools/cartoon_review.py`: five repeatable comparison views, using the existing capture harness.

No model geometry, facial expressions, animation, collision or gameplay changes. No new texture or paid generation. The connected Blender scene was verified and left unchanged; remodelling was unnecessary for this small art-direction adjustment. Old checkpoint files and their evidence were preserved.

### Verification and performance

Release build passed, with existing warnings. All 177 app tests passed; one existing test remained ignored. No new tests were added for these small material/colour adjustments. Logs: `build.txt`, `app-tests.txt`.

All five before/after capture pairs completed with no reported asset/shader errors. Close-up, full Gumdrop and backyard captures were inspected directly. Current-style regression differed at only 18 of 1,024,000 pixels. The brighter sky and greener lawn restore some of the earlier Pop feel; softer cloth and bolder outlines simplify the characters while preserving the newer faces.

11-bot Heist: **127.4 → 129.3 FPS**, mean **7.847 → 7.734 ms**, P95 **8.774 → 8.534 ms**. This is a similar performance level in a short test, not a general improvement guarantee. Stress mesh assets/entities and CPU image data stayed unchanged; materials increased by one through palette-dependent caching. Full results and limits: `PERFORMANCE.md`, `measurements.json`, `evidence/*.metrics.txt`.

Repeat captures from the repository root after building:

```sh
python3 rust/tools/cartoon_review.py --label after
```

Optional `--view normal|gumdrop|yard|stress|current`. For the starting build, use `--binary /tmp/bbq-cartoon-baseline --label before` while that temporary executable exists. The durable starting source is in the recovery archive. Captures use the same camera/simulation frame, and do not write saved settings.

`review.html` contains a comparison slider and an older implemented Pop reference, explicitly labelled as a different camera/scene. HTML script syntax and local images/links were checked. No new browser UI verification is claimed: the browser tool previously rejected local-file URLs, so the gallery is provided for manual opening. Actual game PNGs were inspected directly.

### Verified recovery procedure

1. Inspect Git status and preserve later work. Never reset or extract an archive over the active working tree.
2. Create a separate clean checkout at commit `dc75e0bb7089d2358954022aa642dc536921a829`.
3. Extract `starting-work.tar.gz` into that separate checkout. It contains all modified/new implementation files from the completed lighting/character work at this milestone’s start. Verify `STARTING-FILES.sha256` from its repository root.
4. Apply this folder’s `implementation.patch` with `git apply --check`, then `git apply`.
5. Verify `FILES.sha256` from the repository root, then build.

The starting archive was extracted and every hash checked in a disposable directory. The new patch was checked/applied there and all five resulting files matched the active implementation byte for byte. See `starting-recovery-verification.txt` and `recovery-verification.txt`. No commits, pushes or repository index changes were made.

### Review boundary and next work

Ask Marcus whether this is the desired cartoon direction. Incorporate any adjustments here before proceeding. After review, the next separate milestone is dildo and pool-noodle gravity/motion: inspect mass, drag, rotation, bounce and collision versus visual flex/wobble; preserve aiming, catching and stable settling; compare motion captures and performance. Hit feedback, backyard detail and lobby polish follow later.

This is a small style correction, not a claim of matching the full concept art or improving engagement. The game still uses its existing simplified geometry and surfaces.
