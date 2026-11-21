"""Simplify foliage in the Bush Servo landscape export after the picnic-shade bake."""

import argparse
import json
import struct
from pathlib import Path

import numpy as np

from compact_glb import compact


def coarsen(source, destination):
    raw = source.read_bytes()
    json_size = struct.unpack_from("<I", raw, 12)[0]
    doc = json.loads(raw[20:20 + json_size])
    extras = doc.setdefault("extras", {})
    if extras.get("broaderGumLeaves"):
        raise ValueError("Use the landscape export before this foliage pass.")
    blob = bytearray(raw[28 + json_size:])

    def read_array(index, dtype, columns):
        accessor = doc["accessors"][index]
        view = doc["bufferViews"][accessor["bufferView"]]
        assert "byteStride" not in view and "sparse" not in accessor
        offset = view.get("byteOffset", 0) + accessor.get("byteOffset", 0)
        return np.frombuffer(
            blob, dtype, count=accessor["count"] * columns, offset=offset
        ).reshape(-1, columns).copy()

    def replace_array(index, data, component_type, target):
        blob.extend(bytes(-len(blob) % 4))
        encoded = data.tobytes()
        view = len(doc["bufferViews"])
        doc["bufferViews"].append({
            "buffer": 0, "byteOffset": len(blob), "byteLength": len(encoded), "target": target
        })
        blob.extend(encoded)
        accessor = doc["accessors"][index]
        accessor.update(bufferView=view, byteOffset=0, count=len(data), componentType=component_type)
        if "min" in accessor:
            accessor["min"] = data.min(axis=0).tolist()
            accessor["max"] = data.max(axis=0).tolist()

    rng = np.random.default_rng(17429)
    original = retained = 0
    for primitive in doc["meshes"][0]["primitives"]:
        if not doc["materials"][primitive["material"]]["name"].startswith("Gum foliage "):
            continue
        attributes = primitive["attributes"]
        assert set(attributes) == {"POSITION", "NORMAL"}
        positions = read_array(attributes["POSITION"], "<f4", 3).reshape(-1, 4, 3)
        normals = read_array(attributes["NORMAL"], "<f4", 3).reshape(-1, 4, 3)
        indices = primitive["indices"]
        dtype = "<u2" if doc["accessors"][indices]["componentType"] == 5123 else "<u4"
        current = read_array(indices, dtype, 1).reshape(-1, 6)
        quad = np.array([[0, 1, 2, 0, 2, 3]], dtype=np.uint32)
        assert np.array_equal(current, quad + 4 * np.arange(len(positions))[:, None])

        centres = positions.mean(axis=1)
        canopy = centres[:, 1] > 2.7
        arena = np.any(
            [np.linalg.norm(centres[:, [0, 2]] - centre, axis=1) < 6.5
             for centre in [(12.0, 1.5), (-11.5, 2.0)]], axis=0
        )
        keep = ~canopy | (rng.random(len(positions)) < np.where(arena, 0.63, 0.44))
        original += int(canopy.sum())
        retained += int((canopy & keep).sum())

        # Widen across each original tapered quad. Preserve the branch layout and all scrub.
        width = np.where(arena, 1.50, 2.10)[canopy, None]
        shoulder = (positions[canopy, 1] + positions[canopy, 3]) * 0.5
        for side in [1, 3]:
            positions[canopy, side] = shoulder + (positions[canopy, side] - shoulder) * width

        kept = positions[keep].reshape(-1, 3)
        new_indices = (quad + 4 * np.arange(len(kept) // 4, dtype=np.uint32)[:, None])
        index_dtype, component = ("<u2", 5123) if len(kept) <= 65536 else ("<u4", 5125)
        replace_array(attributes["POSITION"], kept, 5126, 34962)
        replace_array(attributes["NORMAL"], normals[keep].reshape(-1, 3), 5126, 34962)
        replace_array(indices, new_indices.astype(index_dtype).reshape(-1, 1), component, 34963)

    assert 0 < retained < original
    extras["broaderGumLeaves"] = True
    blob.extend(bytes(-len(blob) % 4))
    doc["buffers"][0]["byteLength"] = len(blob)
    header = json.dumps(doc, separators=(",", ":")).encode()
    header += b" " * (-len(header) % 4)
    destination.write_bytes(
        struct.pack("<4sII", b"glTF", 2, 28 + len(header) + len(blob))
        + struct.pack("<I4s", len(header), b"JSON") + header
        + struct.pack("<I4s", len(blob), b"BIN\0") + blob
    )
    compact(destination)
    print(f"Retained {retained:,} of {original:,} canopy leaves.")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("source", type=Path)
    parser.add_argument("destination", type=Path)
    args = parser.parse_args()
    coarsen(args.source, args.destination)
