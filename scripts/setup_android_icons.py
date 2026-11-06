import os
from PIL import Image

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
ROOT_DIR = os.path.dirname(SCRIPT_DIR)
ICON_SRC = os.path.join(ROOT_DIR, "frontend", "assets", "icon-512.png")

if not os.path.exists(ICON_SRC):
    print("Icone base nao encontrado em frontend/assets/icon-512.png")
    exit(0)

img = Image.open(ICON_SRC).convert("RGBA")

densities = {
    "mdpi": 48,
    "hdpi": 72,
    "xhdpi": 96,
    "xxhdpi": 144,
    "xxxhdpi": 192,
}

# Caminho do projeto nativo Android gerado pelo Dioxus
android_res_base = os.path.join(
    ROOT_DIR, "target", "dx", "frontend", "release", "android", "app", "app", "src", "main", "res"
)

if os.path.exists(android_res_base):
    for name, size in densities.items():
        folder = os.path.join(android_res_base, f"mipmap-{name}")
        os.makedirs(folder, exist_ok=True)
        # Remove png se existir para evitar erro de recurso duplicado com webp
        png_path = os.path.join(folder, "ic_launcher.png")
        if os.path.exists(png_path):
            os.remove(png_path)
        resized = img.resize((size, size), Image.Resampling.LANCZOS)
        resized.save(os.path.join(folder, "ic_launcher.webp"), format="WEBP")
        print(f"Icone {name} ({size}x{size}) atualizado com sucesso em WEBP!")
else:
    print("Projeto Android target ainda nao gerado. Rode o build primeiro.")
