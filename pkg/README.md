# Volume-shader-Ultra (vsbm) — Rust + WASM

Rust/WASM rewrite of [volumeshader_bm](https://cznull.github.io/vsbm) by cznull.
Replicates the original WebGL raymarcher 1:1: same fragment shader, same
default Mandelbulb kernel, same camera controls, same CONFIG/APPLY/CANCEL
live-shader-editing panel (including the `CANCLE` typo and startup alert).

## Build

Requires Rust + `wasm-pack`:

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-pack
wasm-pack build --target web --out-dir pkg
```

## Run

Serve the repo root over HTTP (ES modules require HTTP, not `file://`):

```sh
python3 -m http.server 8000
# open http://localhost:8000
```

## Controls

- Left-drag: orbit
- Right-drag: pan
- Wheel: zoom
- 1-finger touch: orbit, 2-finger touch: pan + pinch-zoom
- CONFIG button: toggle kernel editor; APPLY recompiles the fragment shader; CANCLE resets to last-applied kernel

## Notes

- The user-editable `kernal(vec3)` stays a GLSL string compiled at runtime by
  WebGL (Option A from the plan) — this preserves the live-edit feature.
- `wasm-opt` is disabled in `Cargo.toml` to avoid the binaryen download.
- `gl.finish()` is called every frame for fidelity with the original (slower,
  but matches behavior).
