#!/usr/bin/env python3
"""Tiny PNG helper for comparing the browser game with the Rust game (no extra libraries).

    pngtool.py probe  a.png x,y [x,y ...]      colour at some pixels
    pngtool.py diff   a.png b.png [out.png]    average difference, and a picture of where they differ
    pngtool.py side   a.png b.png out.png      the two side by side (left a, right b)
    pngtool.py region a.png b.png x0,y0,x1,y1  average colour of a box in each picture
"""
import struct
import sys
import zlib


def read_png(path):
    data = open(path, "rb").read()
    assert data[:8] == b"\x89PNG\r\n\x1a\n", "not a PNG"
    pos, chunks = 8, []
    while pos < len(data):
        n, t = struct.unpack(">I4s", data[pos:pos + 8])
        chunks.append((t, data[pos + 8:pos + 8 + n]))
        pos += 12 + n
    ihdr = next(c for t, c in chunks if t == b"IHDR")
    w, h, depth, ctype = struct.unpack(">IIBB", ihdr[:10])
    assert depth == 8 and ctype in (2, 6), f"unsupported PNG type {ctype}/{depth}"
    bpp = 4 if ctype == 6 else 3
    raw = zlib.decompress(b"".join(c for t, c in chunks if t == b"IDAT"))
    stride = w * bpp
    rows, prev = [], bytearray(stride)
    i = 0
    for _ in range(h):
        f = raw[i]
        line = bytearray(raw[i + 1:i + 1 + stride])
        i += 1 + stride
        for x in range(stride):
            a = line[x - bpp] if x >= bpp else 0
            b = prev[x]
            c = prev[x - bpp] if x >= bpp else 0
            if f == 1:
                line[x] = (line[x] + a) & 255
            elif f == 2:
                line[x] = (line[x] + b) & 255
            elif f == 3:
                line[x] = (line[x] + (a + b) // 2) & 255
            elif f == 4:
                p = a + b - c
                pa, pb, pc = abs(p - a), abs(p - b), abs(p - c)
                pr = a if pa <= pb and pa <= pc else (b if pb <= pc else c)
                line[x] = (line[x] + pr) & 255
        rows.append(line)
        prev = line
    px = [[tuple(r[x * bpp:x * bpp + 3]) for x in range(w)] for r in rows]
    return w, h, px


def write_png(path, w, h, px):
    raw = b"".join(b"\x00" + bytes(v for p in row for v in p) for row in px)

    def chunk(t, d):
        c = struct.pack(">I", len(d)) + t + d
        return c + struct.pack(">I", zlib.crc32(t + d) & 0xFFFFFFFF)

    open(path, "wb").write(
        b"\x89PNG\r\n\x1a\n"
        + chunk(b"IHDR", struct.pack(">IIBBBBB", w, h, 8, 2, 0, 0, 0))
        + chunk(b"IDAT", zlib.compress(raw, 6))
        + chunk(b"IEND", b"")
    )


def avg(px, x0, y0, x1, y1):
    n = 0
    s = [0, 0, 0]
    for y in range(y0, y1):
        for x in range(x0, x1):
            for k in range(3):
                s[k] += px[y][x][k]
            n += 1
    return tuple(round(v / n, 1) for v in s)


def main(a):
    cmd = a[1]
    if cmd == "probe":
        w, h, px = read_png(a[2])
        for xy in a[3:]:
            x, y = map(int, xy.split(","))
            print(xy, px[y][x])
    elif cmd == "region":
        x0, y0, x1, y1 = map(int, a[4].split(","))
        for p in a[2:4]:
            w, h, px = read_png(p)
            print(p.split("/")[-1], avg(px, x0, y0, x1, y1))
    elif cmd == "diff":
        w, h, p1 = read_png(a[2])
        w2, h2, p2 = read_png(a[3])
        assert (w, h) == (w2, h2), f"sizes differ {w}x{h} vs {w2}x{h2}"
        total = 0
        out = []
        for y in range(h):
            row = []
            for x in range(w):
                d = sum(abs(p1[y][x][k] - p2[y][x][k]) for k in range(3))
                total += d
                v = min(255, d * 2)
                row.append((v, v, v))
            out.append(row)
        print(f"average difference per pixel: {total / (w * h * 3):.2f} of 255")
        if len(a) > 4:
            write_png(a[4], w, h, out)
    elif cmd == "side":
        w, h, p1 = read_png(a[2])
        w2, h2, p2 = read_png(a[3])
        hh = max(h, h2)
        pad = [(255, 0, 255)] * 4
        out = []
        for y in range(hh):
            r1 = p1[y] if y < h else [(0, 0, 0)] * w
            r2 = p2[y] if y < h2 else [(0, 0, 0)] * w2
            out.append(list(r1) + pad + list(r2))
        write_png(a[4], w + 4 + w2, hh, out)
    else:
        print(__doc__)


if __name__ == "__main__":
    main(sys.argv)
