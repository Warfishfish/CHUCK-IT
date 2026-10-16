# Smoko map/mode — Milestone 2
Started 15 October 2026; completed 16 October 2026. Awaiting user review. User approved proceeding, including a Maps / Game modes selector.
Starting HEAD 6dd9397c5ed109a920ebe5a401d72e162b5a968d plus uncommitted Milestone 1. Its modified/new Rust files are preserved in starting-work.tar.gz. Restore only into a separate checkout at HEAD; never overwrite active work. Milestone 1 checkpoint remains in ../2026-10-15-smoko-removal.

Goal: a separately selected Smoko yard with a central marked delivery pad, delivery-only points, F pickup/tap drop/hold throw, fair respawn protection, bots pursuing deliveries, host-authoritative online actions and score. Standard and Heist stay Smoko-free. Review before commit/push.

Plan: add explicit mode and map factory, UI selector and objective explanation; implement ID-based delivery state shared by local player/bots/remote puppets; validate server-side pickup/release and replicate controlled victims; add regression tests and actual UI/game/motion captures; compare performance. Reuse procedural props, movement/carry physics and existing bot combat. No new paid assets or Blender work needed.

Initial rules: +100 per delivery, no other points; only a legitimately grabbed downed/stunned opponent qualifies, owner attribution expires after a short throw window; carriers may be interrupted; recovery cancels delivery eligibility; respawn away from the pad with temporary pickup protection. Constants are kept together for tuning. Full rules and validation will be updated here.

Launch: cd /Users/marcus/Claude/australian-bbq/rust && ~/.cargo/bin/cargo run -p bbq_app --release -- --mode smoko
Starting validation: Milestone 1 passed 598 tests and release build; baseline executable temporarily /tmp/bbq-smoko-m2-before. Final verification is recorded below.

## Progress — 16 October 2026

Implemented GameMode::Smoko, selectable Maps / Game modes buttons, saved mode, Play Smoko label and objective note. Dedicated map factory creates central pad (0,10), radius 3.5; yellow boundary and chairs outside entry lanes. Standard/Heist remain feature-off. Shared map visuals spawn only while Smoko is active.

New smoko_mode.rs owns ID-based carries, throws and pickup protection for host/local/bot/remote participants. F picks up within 2.2m, next tap drops, hold >=0.25s throws with existing carry throw velocity. Eight-second maximum carry; carrier knockdown/stun interrupts. Throw attribution lasts four seconds and is cancelled by recovery/disconnection. Delivery consumes eligibility, awards +100 only, respawns at one of four distant locations and prevents grabs for five seconds. Hits, catches and other bonuses do not score in Smoko.

Networking: guest Carry requests contain only F press/release; host validates distance and state, chooses victim and measures hold time. New Smoko host message updates controlled victim pose/respawn and carrier status. Host ignores incoming movement while victims are controlled. Existing wire fields/format retain standard mode compatibility; peers need the new executable for new messages. Human live multiplayer remains untested.

Bots reuse existing combat plus obstacle-aware objective navigation; Smoko goals replace Heist goals. The full suite now passes 607 tests (194 app, 413 core), with one existing ignored test. Real movement tests cover bot pickup/delivery and thrown trajectory scoring. Network tests cover guest requests, host validation and mode/score serialization. No live human multiplayer session has been performed.

Final carry refinement: hold the passenger low at the right hip (1.2m forward, 1.3m right, 0.1m above carrier ground), with carrier-facing pose. Throw release starts the existing SentFlying animation instead of popping upright. This resolves the obstruction visible in evidence/before-carry-adjustment.png. Final release build passed; the actual-physics capture run logged exactly 100 points and no remaining carry. Unlocked captures now confirm the centre view is clear and the throw awards exactly 100 points.

## Recovery

Starting commit: `6dd9397c5ed109a920ebe5a401d72e162b5a968d`. All Milestone 1 and 2 source changes remain uncommitted. `starting-work.tar.gz` preserves the uncommitted Milestone 1 starting files; `implementation.patch` preserves the combined final source delta from HEAD, including new files. `FILES.sha256` lists all 25 implementation files. Recovery was verified by applying the patch to isolated starting-commit files and comparing all hashes; see recovery-verification.txt.

To recover the completed implementation, create a separate checkout at the starting commit, apply implementation.patch there, then verify FILES.sha256 from that checkout root. Do not extract the starting archive on top of the final patch and never reset or overwrite the active working tree. To recover only the M2 starting state, extract starting-work.tar.gz into a separate checkout at the starting commit instead.

## Files and launch

The exact changed source list is in FILES.sha256. Core mode/map/scoring/message definitions live in bbq_core; app state/rules live in rust/crates/bbq_app/src/smoko_mode.rs; menu.rs contains the selector; yard_scene.rs contains map visuals; online_world.rs and online.rs handle authoritative requests/poses; bots_app.rs supplies delivery goals. Existing movement, animation and procedural prop assets are reused. No Blender asset changes or paid services.

Normal menu:
```sh
cd /Users/marcus/Claude/australian-bbq/rust
~/.cargo/bin/cargo run -p bbq_app --release
```
Choose Solo → Maps / Game modes → Smoko → Play Smoko. Direct launch adds `-- --mode smoko`.

## Final verification — 16 October 2026

- 607 tests passed (194 app, 413 core), one existing ignored; release build passed. See tests.txt and build.txt. No source edits since those checks beyond the capture script's locked-session guard.
- Actual menu, menu launch, map, delivery and stress captures saved in evidence/. Delivery uses a repeatable downed-opponent fixture with real F input, trajectory and scoring. Log confirms `SMOKO_REVIEW final_score=100 carries=0`; all 75 frames are non-black and motion is encoded in delivery.webp. Carry, release and scoring frames inspected. The centre view remains clear with the victim held at the right side.
- Standard/Teams/Heist regression tests remain passing; earlier actual standard/Heist captures are retained in this and the Milestone 1 evidence folders. Live two-person multiplayer is not yet playtested; host validation and wire messages are tested.
- Final Smoko 11-bot run: 8.081ms mean, 7.977ms median, 9.228ms P95, 123.75 FPS; 1/600 intervals over 16.67ms. Asset counts: 2,842 meshes, 886 materials, 6,985 mesh entities, 43,112,856 image CPU bytes.
- Matched Heist comparison: baseline 8.732ms mean / 114.52 FPS; after 7.990ms / 125.16 FPS. Both 0/600 intervals over 16.67ms, identical 2,851 meshes / 893 materials / 7,167 mesh entities / 42,129,816 image CPU bytes. No observed regression; timing differences may be ordinary run-to-run noise. Smoko is a different scene/rule set, not an apples-to-apples speedup comparison.
- Measurements: 1280x800, 11 bots, 300-frame warmup, 600 samples, no vsync, fixed 60Hz simulation. Wall-clock frame intervals, not GPU timestamps.
- Recovery FILES.sha256 rechecked against all 25 implementation files. Full patch independently restored and hash-verified earlier. No commits or pushes.

## Limitations and next step

Review review.html and playtest through Solo → Maps / Game modes → Smoko → Play Smoko. Review carrying feel, bot tactics and spawn fairness; do a live host/guest match before considering online gameplay fully verified. The existing washing line can partially obscure the pad from some approaches, visible in the motion capture. The dedicated layout reuses the backyard terrain and procedural props.

The locked-session interruption is resolved. Invalid black captures remain separately labelled evidence/locked-session-invalid/ and are not performance or visual evidence. Capture script refuses locked-session runs to avoid overwriting valid evidence.

Milestones 1 and 2 remain uncommitted and ready for review. Disable the resume follow-up now; await user review before committing, pushing or starting another milestone.

## User approval — 16 October 2026

User approved committing and pushing both Smoko milestones. This checkpoint is included with the implementation commit; earlier uncommitted/awaiting-review notes describe the verification stage. Resume follow-up is paused. No next milestone is authorized by this approval.
