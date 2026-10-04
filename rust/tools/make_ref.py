#!/usr/bin/env python3
"""Make public/_ref.html: a copy of the browser game with one extra line that lets a
developer reach inside it (window.__ev runs code inside the game's own scope).

Used only to take comparison screenshots and to export the textures for the Rust version.
The copy is git-ignored and is never part of the game.   Run:  python3 rust/tools/make_ref.py
"""
import pathlib

root = pathlib.Path(__file__).resolve().parents[2]
src = (root / "public" / "index.html").read_text()
hook = "window.__ev=function(s){return eval(s);};\n"
marker = "initRoom();\n})();"
assert marker in src, "end of the game script not found"
out = src.replace(marker, "initRoom();\n" + hook + "})();", 1)

# Log every drawn texture so they can be exported as PNG files for the Rust version.
LOG = "(window.__tex=window.__tex||[]).push({c:c,w:w,h:h,rx:rx,ry:ry,d:String(draw).slice(0,90)});"
a = "draw(c.getContext('2d'),w,h);const t=new THREE.CanvasTexture(c);t.wrapS"
assert a in out
out = out.replace(a, "draw(c.getContext('2d'),w,h);" + LOG + "const t=new THREE.CanvasTexture(c);t.wrapS", 1)
b = "draw(c.getContext('2d'),w,h);const t=new THREE.CanvasTexture(c);t.anisotropy=4;return t;}"
assert b in out
out = out.replace(b, "draw(c.getContext('2d'),w,h);" + LOG.replace("rx:rx,ry:ry,", "") + "const t=new THREE.CanvasTexture(c);t.anisotropy=4;return t;}", 1)
f = "const t=new THREE.CanvasTexture(c);t.anisotropy=4;return t;})();"
assert f in out
out = out.replace(f, "(window.__tex=window.__tex||[]).push({c:c,w:256,h:128,d:'FISH'});const t=new THREE.CanvasTexture(c);t.anisotropy=4;return t;})();", 1)

# Same trees and clouds as the Rust game: from just before the trees until the clouds are done,
# the game's random numbers come from the same generator as bbq_core::rng::Rng, seeded with 0xBB0
# (the Rust game's LOOK_SEED), and the trees and clouds ask for them in the same order.
RNG_JS = """
let __rngState=null;
// only the game's own rand() and pick() use the seeded numbers: three.js asks Math.random for
// its object ids, and that would throw the count off
const __rnd=()=>__rngState===null?Math.random():__nextF();
const __M=(1n<<64n)-1n;
function __nextU(){let x=__rngState;x^=x>>12n;x^=(x<<25n)&__M;x^=x>>27n;__rngState=x;return (x*0x2545F4914F6CDD1Dn)&__M;}
function __nextF(){return Number(__nextU()>>40n)/16777216;}
function __seedRng(seed){__rngState=seed===null?null:(((BigInt(seed)^0x9E3779B97F4A7C15n)|1n)&__M);}
"""
out = out.replace("(function(){\n'use strict';", RNG_JS + "(function(){\n'use strict';", 1)
for old, new in (
    ("const rand=(a,b)=>a+Math.random()*(b-a);", "const rand=(a,b)=>a+__rnd()*(b-a);"),
    ("const pick=a=>a[Math.floor(Math.random()*a.length)];", "const pick=a=>a[Math.floor(__rnd()*a.length)];"),
):
    assert old in out, old
    out = out.replace(old, new, 1)
trees = "for(let i=0;i<34;i++){let x,z;do{"
assert trees in out
out = out.replace(trees, "__seedRng(0xBB0);" + trees, 1)
hoist = "// Hills Hoist rotary clothesline"
assert hoist in out
out = out.replace(hoist, "__seedRng(null);" + hoist, 1)

(root / "public" / "_ref.html").write_text(out)
print("wrote public/_ref.html")
