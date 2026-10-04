# Australian BBQ: Rust version

Work in progress. The browser game in `../public` keeps working next to this.
Follow `../RUST_REWRITE_PLAN.md` one phase at a time. Numbers come from `../BEHAVIOUR_SPEC.md`.

## Run it (Mac)

In Terminal, from the project folder:

    cd rust
    cargo run

The first run downloads and compiles Bevy, which takes several minutes. Later runs are fast.
You should see a sky-blue window with a green lawn and a spinning purple cube.

Needs Rust 1.9x or newer (check with `rustc --version`).
