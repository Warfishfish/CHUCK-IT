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
(root / "public" / "_ref.html").write_text(out)
print("wrote public/_ref.html")
