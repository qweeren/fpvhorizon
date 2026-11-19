"""Writes the drone's textures and Blockbench-style (Java block model) models into
hmod/assets/fpvhorizon. One model unit is one centimetre (the game draws them at 0.16 of a
block's 16 units per block, so a unit is 0.01 block); (8, 0, 8) is the bottom centre and the
front is -z (north), where the camera looks.

    python tools/drone_assets.py

The models open in Blockbench (File > Open Model) for hand edits."""

import json
import math
from pathlib import Path

from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parent.parent / "hmod" / "assets" / "fpvhorizon"
TEX = ROOT / "textures" / "block"
MODELS = ROOT / "models" / "drone"


def texture(name, fill, draw=None):
    img = Image.new("RGBA", (16, 16), fill)
    if draw:
        draw(ImageDraw.Draw(img), img)
    TEX.mkdir(parents=True, exist_ok=True)
    img.save(TEX / f"drone_{name}.png")


def carbon(d, img):
    # Twill weave: diagonal light/dark bands.
    for y in range(16):
        for x in range(16):
            v = 34 + (12 if ((x + y) // 2) % 2 else 0) + (4 if (x - y) % 4 == 0 else 0)
            img.putpixel((x, y), (v, v, v + 4, 255))


def battery(d, img):
    d.rectangle([0, 0, 15, 15], fill=(28, 92, 196, 255))
    d.rectangle([0, 0, 15, 1], fill=(56, 132, 230, 255))
    d.rectangle([2, 5, 13, 10], fill=(20, 24, 32, 255))
    d.rectangle([3, 6, 7, 9], fill=(70, 140, 230, 255))
    d.line([0, 15, 15, 15], fill=(16, 60, 140, 255))


def motor(d, img):
    d.rectangle([0, 0, 15, 15], fill=(52, 54, 60, 255))
    for x in range(0, 16, 3):
        d.line([x, 0, x, 15], fill=(84, 88, 96, 255))
    d.rectangle([0, 0, 15, 2], fill=(150, 154, 160, 255))


def lens(d, img):
    d.rectangle([0, 0, 15, 15], fill=(12, 12, 14, 255))
    d.ellipse([3, 3, 12, 12], fill=(30, 36, 60, 255))
    d.ellipse([6, 5, 8, 7], fill=(140, 160, 220, 255))


def camera(d, img):
    d.rectangle([0, 0, 15, 15], fill=(26, 26, 28, 255))
    d.rectangle([0, 0, 15, 0], fill=(60, 60, 64, 255))


def strap(d, img):
    d.rectangle([0, 0, 15, 15], fill=(20, 20, 22, 255))
    for y in range(1, 16, 4):
        d.line([0, y, 15, y], fill=(44, 44, 48, 255))


def prop(d, img):
    d.rectangle([0, 0, 15, 15], fill=(196, 200, 206, 255))
    d.line([0, 15, 15, 15], fill=(150, 154, 160, 255))


def duct(d, img):
    d.rectangle([0, 0, 15, 15], fill=(60, 62, 68, 255))
    d.line([0, 0, 15, 0], fill=(96, 100, 108, 255))
    d.line([0, 15, 15, 15], fill=(40, 40, 44, 255))


def antenna(d, img):
    d.rectangle([0, 0, 15, 15], fill=(36, 36, 40, 255))
    d.rectangle([0, 0, 15, 3], fill=(200, 40, 40, 255))


def box(frm, to, tex, rotation=None, name=None):
    faces = {f: {"texture": f"#{tex}", "uv": [0, 0, 16, 16]} for f in ("north", "south", "east", "west", "up", "down")}
    e = {"from": [round(v, 4) for v in frm], "to": [round(v, 4) for v in to], "faces": faces}
    if rotation:
        e["rotation"] = rotation
    if name:
        e["name"] = name
    return e


def textures(*names):
    t = {n: f"fpvhorizon:block/drone_{n}" for n in names}
    t["particle"] = "fpvhorizon:block/drone_carbon"
    return t


def write(name, elements, tex):
    MODELS.mkdir(parents=True, exist_ok=True)
    model = {"credit": "fpvhorizon (tools/drone_assets.py)", "textures": tex, "elements": elements}
    (MODELS / f"{name}.json").write_text(json.dumps(model, indent=1) + "\n", encoding="utf-8")


def motors(offset, size, height, base):
    out = []
    for sx in (-1, 1):
        for sz in (-1, 1):
            x, z = 8 + sx * offset, 8 + sz * offset
            out.append(box([x - size / 2, base, z - size / 2], [x + size / 2, base + height, z + size / 2], "motor"))
    return out


def x_frame():
    """A 5\" freestyle quad, 245 mm wheelbase: X arms, stack, battery on top, camera in front."""
    k = 24.5 / 2 / math.sqrt(2)  # motor offset along x and z, cm
    arm = 24.5 / 2 + 1.2
    el = [
        box([5, 0, 3], [11, 0.4, 13], "carbon", name="bottom plate"),
        box([8 - arm, 0, 7], [8 + arm, 0.5, 9], "carbon", {"origin": [8, 0, 8], "axis": "y", "angle": 45}, "arm A"),
        box([8 - arm, 0, 7], [8 + arm, 0.5, 9], "carbon", {"origin": [8, 0, 8], "axis": "y", "angle": -45}, "arm B"),
        box([5.5, 2.6, 4], [10.5, 2.9, 12], "carbon", name="top plate"),
        box([6.6, 0.4, 5.5], [9.4, 2.6, 10.5], "strap", name="stack"),
        box([5, 2.9, 4.5], [11, 6.4, 11.5], "battery", name="battery"),
        box([4.8, 2.8, 7.2], [11.2, 6.6, 8.8], "strap", name="strap"),
        box([6.4, 0.5, 2.2], [9.6, 2.6, 4.4], "camera", name="camera"),
        box([7.2, 0.9, 1.7], [8.8, 2.3, 2.2], "lens", name="lens"),
        box([7.7, 2.9, 12.0], [8.3, 3.4, 19.0], "antenna", {"origin": [8, 3, 12], "axis": "x", "angle": 22.5}, "antenna"),
    ]
    el += motors(k, 2.8, 1.6, 0.5)
    write("x_frame", el, textures("carbon", "strap", "battery", "camera", "lens", "antenna", "motor"))


def ducted():
    """A 3.5\" cinewhoop, 142 mm wheelbase: four prop guards round the motors."""
    k = 14.2 / 2 / math.sqrt(2)
    r = 4.8  # duct radius, cm
    wall = 2 * r * math.tan(math.radians(22.5)) + 0.1
    el = [
        box([4.5, 0, 4.5], [11.5, 0.4, 11.5], "carbon", name="bottom plate"),
        box([6.3, 0.4, 6], [9.7, 2.4, 10], "strap", name="stack"),
        box([5.3, 2.4, 5.2], [10.7, 5.2, 10.8], "battery", name="battery"),
        box([5.1, 2.3, 7.3], [10.9, 5.4, 8.7], "strap", name="strap"),
        box([6.6, 0.6, 3.2], [9.4, 2.8, 5.2], "camera", name="camera"),
        box([7.3, 1.0, 2.7], [8.7, 2.4, 3.2], "lens", name="lens"),
    ]
    for sx in (-1, 1):
        for sz in (-1, 1):
            cx, cz = 8 + sx * k, 8 + sz * k
            el.append(box([cx - 1.2, 0, cz - 0.6], [cx + 1.2, 0.4, cz + 0.6], "carbon",
                          {"origin": [cx, 0, cz], "axis": "y", "angle": 45 * sx * sz}))
            for angle in (0, 45):
                rot = {"origin": [cx, 0, cz], "axis": "y", "angle": angle} if angle else None
                # Four walls round the ring; the second set turned 45 degrees closes the octagon.
                el.append(box([cx - wall / 2, 0, cz - r - 0.3], [cx + wall / 2, 2.2, cz - r], "duct", rot))
                el.append(box([cx - wall / 2, 0, cz + r], [cx + wall / 2, 2.2, cz + r + 0.3], "duct", rot))
                el.append(box([cx - r - 0.3, 0, cz - wall / 2], [cx - r, 2.2, cz + wall / 2], "duct", rot))
                el.append(box([cx + r, 0, cz - wall / 2], [cx + r + 0.3, 2.2, cz + wall / 2], "duct", rot))
    el += motors(k, 2.4, 1.4, 0.4)
    write("ducted", el, textures("carbon", "strap", "battery", "camera", "lens", "duct", "motor"))


def prop_blade():
    """One prop blade, 6.35 cm (a 5\" prop's radius) along +x from the hub at (8, 0, 8)."""
    el = [
        box([8, 0, 7.4], [14.35, 0.25, 8.6], "prop", {"origin": [8, 0, 8], "axis": "x", "angle": 22.5}, "blade"),
        box([7.4, -0.2, 7.4], [8.6, 0.6, 8.6], "motor", name="hub"),
    ]
    write("prop", el, textures("prop", "motor"))


if __name__ == "__main__":
    for name, fn in [("carbon", carbon), ("battery", battery), ("motor", motor), ("lens", lens),
                     ("camera", camera), ("strap", strap), ("prop", prop), ("duct", duct), ("antenna", antenna)]:
        texture(name, (0, 0, 0, 255), fn)
    x_frame()
    ducted()
    prop_blade()
    print("wrote", sorted(p.name for p in TEX.iterdir()), sorted(p.name for p in MODELS.iterdir()))
