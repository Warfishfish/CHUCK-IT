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

# ---- plastic: very gentle sun-fade blotches (lighter), fine light scratches, dark specks of dirt
fade = fbm(2, 3)
plastic = 0.90 + 0.08 * (fade - 0.5) + 0.06 * np.clip(fade - 0.6, 0, 1) * 3
plastic += scratches(30, 40, 0.07, 1)
specks = (rng.random((N, N)) > 0.996).astype(float)
plastic -= specks * 0.35
dirt = np.clip(fbm(4, 3) - 0.62, 0, 1) * 1.5
plastic -= dirt * 0.18
save("plastic.png", grey(np.clip(plastic, 0.4, 1.0)))

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
