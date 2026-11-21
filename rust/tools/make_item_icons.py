"""Draw the original inventory artwork at 2x HUD resolution."""
from pathlib import Path

from PIL import Image, ImageDraw

OUT = Path(__file__).resolve().parents[1] / "crates/bbq_app/assets/ui/items"
INK = "#162620"
OUT.mkdir(parents=True, exist_ok=True)


def draw_icon(kind):
    image = Image.new("RGBA", (128, 128))
    d = ImageDraw.Draw(image)

    def oval(box, fill, width=6, outline=INK):
        d.ellipse(box, fill=fill, outline=outline, width=width)

    def box(bounds, radius, fill):
        d.rounded_rectangle(bounds, radius, fill=fill, outline=INK, width=6)

    def polygon(points, fill):
        d.polygon(points, fill=fill, outline=INK, width=5)

    if kind == "teddy":
        for x in (31, 95):
            oval((x - 20, 14, x + 20, 54), "#ad7049")
            oval((x - 9, 25, x + 9, 43), "#e4ae78", outline=None)
        oval((22, 30, 104, 113), "#bb8055")
        oval((43, 73, 85, 103), "#f2c58e", outline=None)
        for x in (47, 81):
            oval((x - 4, 59, x + 4, 68), INK)
        oval((57, 77, 71, 88), INK)
    elif kind == "balloon":
        polygon([(63, 92), (53, 111), (75, 111)], "#238bc8")
        oval((28, 10, 100, 99), "#35c4ef")
        d.arc((40, 21, 81, 61), 192, 264, fill="#d2f7ff", width=7)
    elif kind in ("beer", "rum", "can"):
        if kind == "can":
            box((38, 20, 90, 107), 12, "#edbc30")
            d.ellipse((42, 18, 86, 33), fill="#e3ede7", outline=INK, width=4)
        else:
            box((52, 12, 76, 55), 5, "#326e4f" if kind == "beer" else "#995731")
            box((38, 41, 90, 113), 12, "#448755" if kind == "beer" else "#ad6d3a")
            box((49, 10, 79, 22), 5, "#efca72")
        box((41, 56, 87, 83), 6, "#f5e5ba")
        d.line((51, 69, 77, 69), fill="#945348", width=5)
    elif kind == "fish":
        polygon([(82, 54), (115, 35), (111, 83), (82, 70)], "#7295a4")
        oval((10, 38, 92, 88), "#accbd1")
        oval((24, 55, 34, 65), INK)
        d.arc((40, 48, 73, 81), 291, 64, fill="#648492", width=6)
    elif kind == "steak":
        polygon([(14, 61), (28, 35), (61, 18), (99, 28), (114, 52), (107, 87),
                 (82, 108), (64, 108), (48, 92), (25, 96)], "#bd4660")
        d.line([(23, 64), (35, 42), (61, 29), (97, 36), (105, 51)], fill="#f5c1a2", width=7)
        oval((47, 57, 74, 80), "#f6cfaa", outline="#8b3c45", width=4)
    elif kind in ("noodle", "silicone", "snag"):
        if kind == "silicone":
            box((46, 18, 82, 93), 17, "#ad73dd")
            box((30, 86, 98, 111), 10, "#9553c5")
            d.line((57, 41, 57, 71), fill="#eac9fa", width=5)
        else:
            colour = "#f36da7" if kind == "noodle" else "#a36a49"
            path = [(25, 97), (42, 91), (58, 77), (72, 55), (91, 35), (105, 30)]
            for fill, width in [(INK, 30), (colour, 20)]:
                d.line(path, fill=fill, width=width, joint="curve")
                for x, y in [path[0], path[-1]]:
                    oval((x-width//2, y-width//2, x+width//2, y+width//2), fill, outline=None)
            d.line(path[1:-1], fill="#f2c193", width=3)
    image.resize((64, 64), Image.Resampling.LANCZOS).save(OUT / f"{kind}.png")


for name in ("teddy", "balloon", "beer", "rum", "can", "fish", "steak", "noodle", "silicone", "snag"):
    draw_icon(name)
