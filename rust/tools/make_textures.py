"""Makes the step 2c surface textures (GRAPHICS_2C.md, "Textures"): stylised timber, worn metal,
sun-faded plastic and cracked concrete. Each is 256 x 256, tiles seamlessly, and is mostly a
light neutral grey so the game's own colour for each part still shows through (the texture is
multiplied by the part's colour). Run from the rust/ folder:

    python3 tools/make_textures.py

Needs numpy. Writes crates/bbq_app/assets/textures/{wood,metal,plastic,concrete}.png.
Made by this script, so there is no licence question: the pictures are ours.
"""
import os, struct, zlib
import numpy as np

N = 256
OUT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "crates", "bbq_app", "assets", "textures")
rng = np.random.default_rng(20261005)


def save_normal(name, height, strength):
    """A tangent-space normal map from a height field (light = high), wrapping at the edges so it
    tiles. `strength` is how steep the bumps look."""
    dx = (np.roll(height, -1, axis=1) - np.roll(height, 1, axis=1)) * 0.5 * strength
    dy = (np.roll(height, -1, axis=0) - np.roll(height, 1, axis=0)) * 0.5 * strength
    n = np.stack([-dx, dy, np.ones_like(height)], axis=-1)
    n /= np.linalg.norm(n, axis=-1, keepdims=True)
    save(name, n * 0.5 + 0.5)


def save(name, rgb):
    """rgb: N x N x 3 floats 0..1 -> PNG (RGBA, opaque)."""
    a = np.clip(rgb, 0, 1)
    px = (a * 255 + 0.5).astype(np.uint8)
    rows = b"".join(b"\x00" + bytes(np.concatenate([px[y], np.full((N, 1), 255, np.uint8)], axis=1).tobytes()) for y in range(N))
    def chunk(t, d):
        c = struct.pack(">I", len(d)) + t + d
        return c + struct.pack(">I", zlib.crc32(t + d) & 0xFFFFFFFF)
    png = b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", N, N, 8, 6, 0, 0, 0)) + chunk(b"IDAT", zlib.compress(rows, 9)) + chunk(b"IEND", b"")
    with open(os.path.join(OUT, name), "wb") as f:
        f.write(png)
    print("wrote", name, a.mean().round(3))


def noise(cells, sx=1, sy=1):
    """Smooth value noise that tiles: `cells` lattice points across (sx, sy stretch it)."""
    cx, cy = max(1, cells * sx), max(1, cells * sy)
    lat = rng.random((cy, cx))
    y = np.arange(N) * cy / N
    x = np.arange(N) * cx / N
    y0 = np.floor(y).astype(int); x0 = np.floor(x).astype(int)
    ty = (y - y0)[:, None]; tx = (x - x0)[None, :]
    ty = ty * ty * (3 - 2 * ty); tx = tx * tx * (3 - 2 * tx)
    y1 = (y0 + 1) % cy; x1 = (x0 + 1) % cx
    a = lat[y0][:, x0]; b = lat[y0][:, x1]; c = lat[y1][:, x0]; d = lat[y1][:, x1]
    return a + (b - a) * tx + (c - a) * ty + (a - b - c + d) * tx * ty


def fbm(base, octaves=4, sx=1, sy=1):
    v = np.zeros((N, N)); amp = 1.0; tot = 0.0
    for o in range(octaves):
        v += noise(base * 2 ** o, sx, sy) * amp
        tot += amp; amp *= 0.5
    return v / tot


def scratches(count, length, bright, width=1, horizontal_bias=0.0):
    """Thin straight-ish scratch lines that wrap round the edges. Returns a mask 0..1."""
    m = np.zeros((N, N))
    for _ in range(count):
        x, y = rng.random() * N, rng.random() * N
        a = rng.normal(0, 0.3) if rng.random() < horizontal_bias else rng.random() * np.pi
        L = rng.integers(length // 3, length)
        for t in range(L):
            xi, yi = int(x + np.cos(a) * t) % N, int(y + np.sin(a) * t) % N
            for w in range(width):
                m[(yi + w) % N, xi] = max(m[(yi + w) % N, xi], 1.0 - t / L * 0.5)
            a += rng.normal(0, 0.02)
    return m * bright


def grey(v, tint=(1.0, 1.0, 1.0)):
    return np.stack([v * tint[0], v * tint[1], v * tint[2]], axis=-1)


# ---- timber: crisp growth-ring lines running along u (wandering a little), a light-to-dark
# gradient inside each ring, fine fibres, knots and a few dark scratches; a slightly warm cast
warp = fbm(2, 3, sx=1, sy=2) * 1.6
yy = np.arange(N)[:, None] * np.ones((1, N))
xx = np.ones((N, 1)) * np.arange(N)[None, :]
phase = yy / N * 11 + warp + 0.25 * np.sin(xx / N * 2 * np.pi * 2)
f = phase - np.floor(phase)
line = np.exp(-(f / 0.05) ** 2) + np.exp(-((1 - f) / 0.05) ** 2)
fibre = fbm(16, 2, sx=6, sy=1)
wood = 0.92 - 0.10 * f - 0.24 * line + 0.07 * (fibre - 0.5)
# knots: dark ellipses with rings round them
for _ in range(3):
    kx, ky = rng.random() * N, rng.random() * N
    dx = (xx - kx + N / 2) % N - N / 2
    dy = (yy - ky + N / 2) % N - N / 2
    r = np.sqrt((dx / 2.6) ** 2 + dy ** 2)
    wood -= 0.30 * np.exp(-(r / 4.0) ** 2) + 0.07 * np.exp(-(r / 11) ** 2) * (0.5 + 0.5 * np.sin(r * 1.4))
wood -= scratches(14, 70, 0.12, 1, horizontal_bias=0.8)
wood = np.clip(wood, 0.42, 1.0)
save("wood.png", grey(wood, (1.0, 0.96, 0.9)))
save_normal("wood_n.png", wood - 0.8 * line * 0.15, 6.0)

# ---- metal: brushed streaks, scratches (bright), grime blotches, a little rust here and there
streak = fbm(16, 3, sx=8, sy=1)  # long in x
grime = fbm(3, 4)
metal = 0.86 + 0.08 * (streak - 0.5) - 0.18 * np.clip(grime - 0.55, 0, 1) * 2
metal += scratches(20, 60, 0.08, 1)
rgb = grey(np.clip(metal, 0.4, 1.0))
rust = np.clip((fbm(5, 4) - 0.68) * 5, 0, 1) * np.clip(grime * 1.5, 0, 1)
rust_col = np.array([0.62, 0.36, 0.18])
rgb = rgb * (1 - rust[..., None] * 0.8) + rust_col * rust[..., None] * 0.8
save("metal.png", rgb)
save_normal("metal_n.png", metal + rust * 0.3, 3.0)

# ---- plastic: very gentle sun-fade blotches (lighter), fine light scratches, dark specks of dirt
fade = fbm(2, 3)
plastic = 0.90 + 0.08 * (fade - 0.5) + 0.06 * np.clip(fade - 0.6, 0, 1) * 3
plastic += scratches(30, 40, 0.07, 1)
specks = (rng.random((N, N)) > 0.996).astype(float)
plastic -= specks * 0.35
dirt = np.clip(fbm(4, 3) - 0.62, 0, 1) * 1.5
plastic -= dirt * 0.18
save("plastic.png", grey(np.clip(plastic, 0.4, 1.0)))
save_normal("plastic_n.png", plastic, 2.0)

# ---- concrete: aggregate speckle, stains, dark cracks that wander
base = 0.82 + 0.10 * (fbm(6, 4) - 0.5)
agg = rng.random((N, N))
base += np.where(agg > 0.985, 0.10, 0) - np.where(agg < 0.015, 0.12, 0)
stain = np.clip(fbm(2, 4) - 0.6, 0, 1) * 1.6
base -= stain * 0.18
crack = np.zeros((N, N))
for _ in range(5):
    x, y = rng.random() * N, rng.random() * N
    a = rng.random() * 2 * np.pi
    for t in range(rng.integers(60, 160)):
        crack[int(y) % N, int(x) % N] = 1
        if rng.random() < 0.4:
            crack[int(y + 1) % N, int(x) % N] = 0.6
        x += np.cos(a) * 1.0; y += np.sin(a) * 1.0
        a += rng.normal(0, 0.35)
        if rng.random() < 0.02:
            a += rng.choice([-1, 1]) * 0.9  # a branch kink
base -= crack * 0.38
save("concrete.png", grey(np.clip(base, 0.35, 1.0), (1.0, 0.99, 0.96)))
save_normal("concrete_n.png", base, 5.0)


# ---- decals (white with an alpha shape; the game tints them): stain, splat, scorch
def save_rgba(name, alpha):
    a = np.clip(alpha, 0, 1)
    rgba = np.zeros((N, N, 4), np.uint8)
    rgba[..., :3] = 255
    rgba[..., 3] = (a * 255 + 0.5).astype(np.uint8)
    rows = b"".join(b"\x00" + rgba[y].tobytes() for y in range(N))
    def chunk(t, d):
        c = struct.pack(">I", len(d)) + t + d
        return c + struct.pack(">I", zlib.crc32(t + d) & 0xFFFFFFFF)
    png = b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", N, N, 8, 6, 0, 0, 0)) + chunk(b"IDAT", zlib.compress(rows, 9)) + chunk(b"IEND", b"")
    with open(os.path.join(OUT, name), "wb") as f:
        f.write(png)
    print("wrote", name)


cx = (np.arange(N)[None, :] - N / 2) / (N / 2)
cy = (np.arange(N)[:, None] - N / 2) / (N / 2)
rad = np.sqrt(cx ** 2 + cy ** 2)
ang = np.arctan2(cy, cx)

# stain: an irregular blotch with a darker rim, a few satellite drops, fading to nothing at the edge
wob = 0.12 * np.sin(ang * 3 + 1.1) + 0.08 * np.sin(ang * 5 + 0.3) + 0.05 * np.sin(ang * 9 + 2.0)
edge = 0.62 + wob + 0.06 * (fbm(6, 3) - 0.5)
stain = np.clip((edge - rad) / 0.18, 0, 1)
stain *= 0.75 + 0.25 * fbm(5, 3)
for _ in range(7):
    a0 = rng.random() * 2 * np.pi; r0 = rng.uniform(0.72, 0.9); s0 = rng.uniform(0.025, 0.06)
    d = np.sqrt((cx - np.cos(a0) * r0) ** 2 + (cy - np.sin(a0) * r0) ** 2)
    stain = np.maximum(stain, np.clip((s0 - d) / 0.02, 0, 1) * 0.8)
save_rgba("decal_stain.png", stain * (1 - np.clip(rad - 0.9, 0, 0.1) * 10))

# splat: a small centre with spokes of drops flung outwards (bird droppings, sauce, a smashed can)
splat = np.clip((0.2 - rad) / 0.05, 0, 1) * 0.95
for k in range(14):
    a0 = rng.random() * 2 * np.pi
    for j in range(rng.integers(2, 5)):
        r0 = rng.uniform(0.25, 0.85); s0 = rng.uniform(0.02, 0.07) * (1.2 - r0)
        d = np.sqrt((cx - np.cos(a0 + rng.normal(0, 0.12)) * r0) ** 2 + (cy - np.sin(a0 + rng.normal(0, 0.12)) * r0) ** 2)
        splat = np.maximum(splat, np.clip((s0 - d) / 0.015, 0, 1) * 0.9)
save_rgba("decal_splat.png", splat * (rad < 0.95))

# scorch: a soft dark smudge with a smoky streak
sc = np.clip((0.7 - np.sqrt((cx / 1.0) ** 2 + (cy / 0.75) ** 2)) / 0.5, 0, 1) ** 1.2
sc *= 0.6 + 0.4 * fbm(4, 4)
save_rgba("decal_scorch.png", sc * (rad < 0.98))


# ---- gum-tree bark: pale smooth bark with long vertical streaks and patches where the old bark
# has peeled away (tan and grey), running up the trunk (v)
streaks = fbm(3, 3, sx=6, sy=1)          # narrow across, long up the trunk
patches = np.clip((fbm(3, 4) - 0.52) * 4.0, 0, 1)
fine = fbm(10, 2, sx=4, sy=1)
bark = 0.86 + 0.08 * (streaks - 0.5) + 0.05 * (fine - 0.5)
rgb = grey(bark, (1.0, 0.98, 0.93))
tan = np.array([0.72, 0.58, 0.42])
greyp = np.array([0.62, 0.62, 0.6])
mix = np.clip(fbm(2, 2) - 0.5, 0, 1)[..., None] * 2
peel = tan * (1 - mix) + greyp * mix
rgb = rgb * (1 - patches[..., None] * 0.85) + peel * patches[..., None] * 0.85
# a dark edge round each peeled patch
edge = np.clip(1 - np.abs(patches - 0.5) * 4, 0, 1)
rgb *= (1 - edge[..., None] * 0.18)
save("bark.png", rgb)
save_normal("bark_n.png", bark - patches * 0.08, 4.0)


# ---- clay: a soft mottled surface with fine pores and a few thumb-print swirls; grey, so each
# character's own colour tints it. Used on the characters in the "clay" style.
mott = fbm(5, 4)
pores = (rng.random((N, N)) > 0.985).astype(float)
swirl = np.zeros((N, N))
for _ in range(5):
    sx, sy = rng.random() * N, rng.random() * N
    dx = (np.arange(N)[None, :] - sx + N / 2) % N - N / 2
    dy = (np.arange(N)[:, None] - sy + N / 2) % N - N / 2
    r = np.sqrt(dx * dx + dy * dy)
    swirl += np.exp(-(r / 22) ** 2) * np.sin(r * 0.55) * 0.4
mott_n = (mott - mott.mean()) / mott.std()
clay = 0.88 + 0.07 * mott_n - pores * 0.10 + 0.06 * swirl
save("clay.png", grey(np.clip(clay, 0.6, 1.0), (1.0, 0.98, 0.95)))
save_normal("clay_n.png", 0.5 * (mott - 0.5) + 0.5 * swirl + 0.4 * fbm(14, 2) - pores * 0.35, 3.0)

# ---- fabric: a fine woven cloth (towels, washing, the umbrella). Light grey, tiles, with a
# threads-over-and-under weave, a few soft creases and slubs.
threads = 48
wx = np.sin(xx / N * 2 * np.pi * threads)
wy = np.sin(yy / N * 2 * np.pi * threads)
weave = 0.5 + 0.25 * wx * wy + 0.15 * (np.abs(wx) - np.abs(wy)) * 0.5
crease = fbm(3, 3, sx=2, sy=1)
slub = fbm(24, 2, sx=1, sy=6)
fabric = 0.90 + 0.07 * (weave - 0.5) - 0.05 * np.clip(crease - 0.55, 0, 1) * 2 - 0.03 * (slub - 0.5)
save("fabric.png", grey(np.clip(fabric, 0.6, 1.0), (1.0, 0.99, 0.97)))
save_normal("fabric_n.png", weave * 0.6 + 0.3 * crease + 0.1 * slub, 1.6)

# ---- packed maps for wood and metal: red = ambient occlusion (dark in the grain lines, knots and
# grime), green = roughness, blue = metallic (0: the game has no sky reflections to give metal).
# The material multiplies them by its own roughness, so these only vary it across the surface.
def save_orm(name, ao, rough):
    a = np.stack([np.clip(ao, 0, 1), np.clip(rough, 0, 1), np.zeros_like(ao)], axis=-1)
    save(name, a)

wn = (wood - 0.42) / 0.58
save_orm("wood_orm.png", 0.55 + 0.45 * wn, 0.80 + 0.20 * (1 - wn))
mn = np.clip((metal - 0.4) / 0.6, 0, 1)
save_orm("metal_orm.png", 0.60 + 0.40 * mn - 0.2 * rust, 0.55 + 0.45 * (1 - mn) + 0.35 * rust)
