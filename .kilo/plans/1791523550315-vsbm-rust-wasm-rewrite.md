# Plan: Rewrite volumeshader_bm (vsbm) in Rust + WASM

## Goal

Replicate https://cznull.github.io/vsbm (source: `vsbm.html` in github.com/cznull/cznull.github.io) as a Rust/WASM application. Behavior, visuals, and quirks must match the original 1:1.

## Resolved Decisions

1. **Kernel handling (Option A):** Keep runtime GLSL compilation. The user-editable `kernal(vec3)` stays a GLSL string (default = Mandelbulb kernel from original). Rust recompiles the fragment shader on APPLY and alerts the info log on failure. No in-browser Rust compilation.
2. **Framework (Option A):** Plain `wasm-bindgen` + `web-sys`. No yew/leptos/gloo. `wasm-pack` builds to `pkg/`.
3. **Integration (Option A):** Static `index.html` shell (original CSS/IDs verbatim) + `wasm-pack build --target web` ES-module script. Rust owns all logic; DOM is a static shell.
4. **Fidelity (Option A):** Replicate everything verbatim, including: startup `alert("trigonometric and inverse trigonometric functions test.\ncreated by cznull@bilibili")`, the `CANCLE` typo, auto-rotation `ang1 += 0.01` per frame, `gl.finish()` per frame, 1024×1024 canvas CSS-scaled by `min(vw,vh)/1024`, and all camera constants.

## Architecture

```
/workspace/app
├── index.html          # static shell: canvas#c1, button#btn, div#config,
│                       # textarea#kernel, buttons#apply/#cancle; original CSS
├── Cargo.toml          # crate-type = ["cdylib"], deps: wasm-bindgen, web-sys, js-sys, wasm-bindgen-futures
├── src/lib.rs          # all app logic (see modules below)
└── pkg/                # wasm-pack output (gitignored)
```

## Implementation Tasks (ordered)

1. **Scaffold**: `Cargo.toml` with `wasm-bindgen`, `web-sys` (features: `Window`, `Document`, `HtmlElement`, `HtmlCanvasElement`, `HtmlTextAreaElement`, `HtmlButtonElement`, `WebGlRenderingContext`, `WebGlProgram`, `WebGlShader`, `WebGlBuffer`, `WebGlUniformLocation`, `Event`, `MouseEvent`, `WheelEvent`, `TouchEvent`, `KeyboardEvent`), `js-sys`, `wasm-bindgen-futures`.

2. **`index.html`**: copy original HTML/CSS verbatim (body `#131115`, canvas `#fbf7fe`, `#btn` fixed top-left, `#c1` fixed top-left, `#main` `transform-origin: 0 0`, `#config` `top:30px; display:none`, `overflow-x/y:hidden`, `user-scalable=no`). Replace inline `<script>` with `<script type="module">import init from './pkg/vsbm.js'; init();</script>`.

3. **State struct** (mirrors original globals): `cx, cy` (viewport), `len=1.6`, `ang1=2.8`, `ang2=0.4`, `cenx/ceny/cenz=0.0`, `mx,my,mx1,my1`, `lasttimen`, `ml,mr,mm`, `kernel: String` (default Mandelbulb GLSL, exact string from original), WebGL handles (`program`, `frag_shader`, `vert_shader`, `buffer`, uniform/attrib locations).

4. **WebGL init**: create program, compile vertex shader (original `VSHADER_SOURCE` verbatim), compile fragment shader (`FSHADER_SOURCE + kernel` verbatim), link, get locations for `position`, `right`, `forward`, `up`, `origin`, `x`, `y`, `len`; upload the 6-vertex fullscreen triangle buffer; `viewport(0,0,1024,1024)`.

5. **`draw()`**: set uniforms exactly as original (`x = cx*2/(cx+cy)`, `y = cy*2/(cx+cy)`, `len`, `origin = (len·cos(ang1)·cos(ang2)+cenx, len·sin(ang2)+ceny, len·sin(ang1)·cos(ang2)+cenz)`, `right = (sin(ang1), 0, -cos(ang1)`, `up = (-sin(ang2)·cos(ang1), cos(ang2), -sin(ang2)·sin(ang1))`, `forward = (-cos(ang1)·cos(ang2), -sin(ang2), -sin(ang1)·cos(ang2))`), `drawArrays(TRIANGLES, 0, 6)`, `finish()`.

6. **Render loop**: `requestAnimationFrame` closure; each frame `ang1 += 0.01; draw();`.

7. **Events** (all on `document`/`window`, matching original):
   - `mousedown`/`mouseup`: left→`ml`, right→`mr`, track `mx,my`.
   - `mousemove`: left-drag orbit (`ang1 += dx*0.002`, `ang2 += dy*0.002`); right-drag pan with original projection math (`l = len*4/(cx+cy)`); set `mm=1` on movement.
   - `mousewheel`: `preventDefault`, `len *= exp(-0.001*wheelDelta)`.
   - `touchstart`/`touchend`/`touchmove`: 1-finger orbit, 2-finger pan + pinch (`l = len*2/(cx+cy)`, pinch ratio from distance+1.0), `preventDefault` on move.
   - `contextmenu`: `preventDefault()` only when `mm==1`.
   - `resize` + initial: `cx=cy=min(vw,vh)`, `#main` size 1024px, `transform: scale(cx/1024, cy/1024)`.

8. **CONFIG UI**: `#btn` click toggles text `CONFIG`↔`HIDE` and `#config` display `inline`↔`none`. `#apply` click: read textarea → `kernel`, recompile fragment shader, relink, re-fetch all locations, `alert(infoLog)` on link failure. `#cancle` click: reset textarea to `kernel`. On init: set textarea value to default kernel, fire startup alert.

9. **Build/serve**: `wasm-pack build --target web --out-dir pkg`. Serve repo root statically (e.g. `python3 -m http.server`) — ES modules require HTTP, not `file://`.

## Risks / Notes

- **GLSL string fidelity**: the fragment shader and default kernel must be copied character-for-character (escaping `\n` in Rust string literals). Any drift changes rendering.
- **`web-sys` API drift**: `WebGlRenderingContext` methods are fallible (`Result`); unwrap or log. `wheelDelta` is non-standard but available via `WheelEvent::wheel_delta()` in web-sys.
- **`gl.finish()` per frame** is slow but required for fidelity.
- **Touch + mouse both active**: original doesn't guard against this; replicate as-is.
- **No tests possible for visuals**; validation is manual browser comparison against the live site.

## Validation

1. `wasm-pack build --target web` succeeds with no warnings that break the build.
2. Serve repo root; open in Chrome/Firefox; startup alert appears.
3. Visual diff vs https://cznull.github.io/vsbm: same default Mandelbulb render, same auto-rotation speed, same background colors.
4. Left-drag orbits, right-drag pans, wheel zooms, two-finger touch works.
5. CONFIG button toggles panel; editing kernel + APPLY recompiles (try a trivial kernel like `return 1.0-length(ver);`); invalid GLSL triggers alert with compile error; CANCEL restores last-applied kernel.
6. Resize window: canvas stays square, scales to `min(vw,vh)`.

## Out of Scope

- Pure-Rust/WASM compute raymarcher (rejected — loses live shader editing).
- UI modernization, typo fixes, performance optimization.
- Other pages in the cznull.github.io repo (only `/vsbm` is in scope).
