"""Paint tileable corrugation and weatherboards for the Backyard house."""
from pathlib import Path
import struct
import zlib

import numpy as np

OUTPUT = Path(__file__).resolve().parents[1] / "crates/bbq_app/assets/textures"
SIZE = 512
y, x = np.mgrid[:SIZE, :SIZE] / SIZE * np.pi * 2
grain = np.sin(11 * x + 8 * y) * np.sin(31 * y - 13 * x)
weather = np.sin(3 * x - y) * np.sin(5 * y + x)


def save(name, base, lighting):
    rgb = np.clip(np.rint(np.asarray(base) + lighting[:, :, None]), 0, 255).astype("uint8")

    def chunk(tag, data):
        return struct.pack(">I", len(data)) + tag + data + struct.pack(">I", zlib.crc32(tag + data))

    data = b"".join(b"\0" + row.tobytes() for row in rgb)
    png = b"\x89PNG\r\n\x1a\n"
    png += chunk(b"IHDR", struct.pack(">IIBBBBB", SIZE, SIZE, 8, 2, 0, 0, 0))
    png += chunk(b"IDAT", zlib.compress(data, 9)) + chunk(b"IEND", b"")
    (OUTPUT / name).write_bytes(png)


ribs = np.cos(11 * x)
save("postcard_roof.png", [105, 159, 145], 19 * ribs + 5 * np.sin(22 * x) + 5 * weather + 2 * grain)

course = (7 * y / (np.pi * 2)) % 1
edge = np.maximum(0, 1 - course * 18)
lip = np.maximum(0, 1 - np.abs(course - 0.12) * 13)
save("postcard_siding.png", [230, 208, 168], -32 * edge + 12 * lip + 5 * weather + 2 * grain)
