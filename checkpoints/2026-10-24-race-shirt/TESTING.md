# Testing the update

```sh
cd /Users/marcus/Claude/australian-bbq/rust
~/.cargo/bin/cargo run -p bbq_app --release -- --menu
```

Main menu → Play solo → Maps / Game modes → Race → Racer → Play.

- Walk near another grounded racer; the F pickup prompt appears. Press F to collect them. Tap F again to drop, or hold F for at least a quarter-second and release to throw. Automatic escape after three seconds; five seconds of pickup immunity after release.
- Face a nearby racer and press R to tug their shorts. The clothing stays on. This briefly slows them, without scoring. Victims get six seconds of immunity from another tug.
- Run off the lane. After a narrow forgiving verge, grass slows movement to 55%. Return to the gold ring near your departure point; checkpoint progress then resumes. Carrying or throwing cannot skip the course.
- Choose Sideline Thrower to check the unchanged hit-streak/upgrade game.
- In any mode, look at another character from behind to see their shirt name.

Online players must all reload/build protocol 15. Start a yard through With mates → Host a yard; choose Race in the lobby setup.

No commit or push is included in this update.
