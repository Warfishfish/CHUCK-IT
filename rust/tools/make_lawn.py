"""Blender: deterministic, seamless 512px lawn albedo and normal maps for Pop.

Run with Blender's Python or the connected Blender tool. Creates image/material
datablocks only; does not change the user's scene, objects, selection or render settings.
No external packages, downloads or generation credits. Existing game assets are untouched.
"""
import math
from pathlib import Path
import random

import bpy

SIZE = 512
OUT = Path(__file__).resolve().parents[1] / 'crates/bbq_app/assets/textures'
rng = random.Random(20261013)


def noise_grid(cells):
    lattice = [[rng.random() for _ in range(cells)] for _ in range(cells)]
    values = []
    for y in range(SIZE):
        fy = y * cells / SIZE
        iy = int(fy)
        ty = fy - iy
        ty = ty * ty * (3 - 2 * ty)
        for x in range(SIZE):
            fx = x * cells / SIZE
            ix = int(fx)
            tx = fx - ix
            tx = tx * tx * (3 - 2 * tx)
            a, b = lattice[iy][ix], lattice[iy][(ix + 1) % cells]
            c, d = lattice[(iy + 1) % cells][ix], lattice[(iy + 1) % cells][(ix + 1) % cells]
            values.append((a + (b - a) * tx) * (1 - ty) + (c + (d - c) * tx) * ty)
    return values


broad, fine = noise_grid(8), noise_grid(48)
height = [0.45 * b + 0.25 * f + rng.uniform(-0.05, 0.05) for b, f in zip(broad, fine)]
# Short bent blades, wrapped at tile edges. No directional mowing bands or baked shadows.
blades = [0.0] * (SIZE * SIZE)
for _ in range(22000):
    x, y = rng.randrange(SIZE), rng.randrange(SIZE)
    angle = rng.uniform(0, math.tau)
    length = rng.randrange(2, 8)
    strength = rng.uniform(-0.3, 0.4)
    for step in range(length):
        angle += 0.06
        px = int(x + math.cos(angle) * step) % SIZE
        py = int(y + math.sin(angle) * step) % SIZE
        index = py * SIZE + px
        weight = math.sin(math.pi * (step + 0.5) / length)
        blades[index] += strength * weight
        height[index] += abs(strength) * weight * 0.28

colour, normals = [], []
for y in range(SIZE):
    for x in range(SIZE):
        i = y * SIZE + x
        variation = 1.0 + 0.06 * (broad[i] - 0.5) + 0.13 * (fine[i] - 0.5) + blades[i] * 0.3
        straw = max(0.0, broad[i] - 0.56) * 0.04
        colour.extend((min(1.0, 0.37 * variation + straw), min(1.0, 0.46 * variation + straw * 0.5), min(1.0, 0.24 * variation), 1.0))
        dx = (height[y * SIZE + (x + 1) % SIZE] - height[y * SIZE + (x - 1) % SIZE]) * 0.65
        dy = (height[((y + 1) % SIZE) * SIZE + x] - height[((y - 1) % SIZE) * SIZE + x]) * 0.65
        inv = 1.0 / math.sqrt(dx * dx + dy * dy + 1.0)
        normals.extend((0.5 - dx * inv * 0.5, 0.5 - dy * inv * 0.5, 0.5 + inv * 0.5, 1.0))

for name, pixels in [('lawn_afternoon.png', colour), ('lawn_afternoon_n.png', normals)]:
    image = bpy.data.images.get(name) or bpy.data.images.new(name, width=SIZE, height=SIZE, alpha=True)
    image.colorspace_settings.name = 'Non-Color'
    image.pixels.foreach_set(pixels)
    image.filepath_raw = str(OUT / name)
    formats = [item.identifier for item in image.bl_rna.properties['file_format'].enum_items]
    assert 'PNG' in formats
    image.file_format = 'PNG'
    image.save()
    print(f'Saved {image.filepath_raw}: {SIZE}x{SIZE}')

material = bpy.data.materials.get('BBQ Afternoon Lawn') or bpy.data.materials.new('BBQ Afternoon Lawn')
material.use_nodes = True
nodes = material.node_tree.nodes
nodes.clear()  # This generator owns this material; reruns must not accumulate nodes.
shader = nodes.new('ShaderNodeBsdfPrincipled')
output = nodes.new('ShaderNodeOutputMaterial')
material.node_tree.links.new(shader.outputs['BSDF'], output.inputs['Surface'])
shader.inputs['Roughness'].default_value = 0.9
albedo = nodes.new('ShaderNodeTexImage')
albedo.image = bpy.data.images['lawn_afternoon.png']
normal = nodes.new('ShaderNodeTexImage')
normal.image = bpy.data.images['lawn_afternoon_n.png']
normal_map = nodes.new('ShaderNodeNormalMap')
material.node_tree.links.new(albedo.outputs['Color'], shader.inputs['Base Color'])
material.node_tree.links.new(normal.outputs['Color'], normal_map.inputs['Color'])
material.node_tree.links.new(normal_map.outputs['Normal'], shader.inputs['Normal'])
print('Created BBQ Afternoon Lawn material. Current scene unchanged.')
