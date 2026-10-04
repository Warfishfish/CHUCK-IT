#!/usr/bin/env python3
"""Fit  pixel = a + b*n.y + s*max(n.L, 0)  to a picture of the grey calibration ball,
so the browser's lighting and the Rust lighting can be compared number for number.

    fit_light.py ball.png      (camera at (0,30,2), ball radius 0.8 at (0,30,-1), fov 45, 800 x 600)
"""
import sys
import numpy as np
sys.path.insert(0, __file__.rsplit("/", 1)[0])
import pngtool

ALBEDO = 0x66 / 255.0
SUN = np.array([24.0, 40.0, 20.0])
SUN = SUN / np.linalg.norm(SUN)


def fit(path):
    w, h, px = pngtool.read_png(path)
    img = np.array(px, dtype=float) / 255.0 / ALBEDO  # the ball is grey 0x66 = 0.4
    cam = np.array([0.0, 30.0, 2.0])
    centre = np.array([0.0, 30.0, -1.0])
    r = 0.8
    f = (h / 2) / np.tan(np.radians(45) / 2)
    rows, ys, cols = [], [], []
    for y in range(0, h, 3):
        for x in range(0, w, 3):
            d = np.array([(x + 0.5 - w / 2) / f, -(y + 0.5 - h / 2) / f, -1.0])
            d /= np.linalg.norm(d)
            oc = cam - centre
            b = oc @ d
            c = oc @ oc - r * r
            disc = b * b - c
            if disc <= 0:
                continue
            t = -b - np.sqrt(disc)
            n = (cam + d * t - centre) / r
            if n[2] < 0.15:  # skip the very edge
                continue
            rows.append([1.0, n[1], max(n @ SUN, 0.0)])
            if img[y, x].max() > 2.3:  # clipped pixels would bend the fit
                rows.pop()
                continue
            cols.append(img[y, x])
    A = np.array(rows)
    Y = np.array(cols)
    coef, res, *_ = np.linalg.lstsq(A, Y, rcond=None)
    pred = A @ coef
    err = np.abs(pred - Y).mean()
    return coef, err


if __name__ == "__main__":
    coef, err = fit(sys.argv[1])
    names = ["ambient a", "up b (n.y)", "sun s (n.L)"]
    print(f"{sys.argv[1].split('/')[-1]}  (mean fit error {err:.4f})")
    for i, nme in enumerate(names):
        print(f"  {nme:12s} R {coef[i][0]:.3f}  G {coef[i][1]:.3f}  B {coef[i][2]:.3f}")
