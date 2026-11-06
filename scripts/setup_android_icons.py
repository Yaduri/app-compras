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
    # 1. Remove os arquivos vetoriais padrão do robô verde do Android
    anydpi_dir = os.path.join(android_res_base, "mipmap-anydpi-v26")
    if os.path.exists(anydpi_dir):
        import shutil
        shutil.rmtree(anydpi_dir)
        print("Removido mipmap-anydpi-v26 do robozinho verde.")

    for bg_file in [
        os.path.join(android_res_base, "drawable", "ic_launcher_background.xml"),
        os.path.join(android_res_base, "drawable-v24", "ic_launcher_foreground.xml"),
    ]:
        if os.path.exists(bg_file):
            os.remove(bg_file)

    # 2. Gera os ícones minimalistas padrão e redondos em WEBP
    for name, size in densities.items():
        folder = os.path.join(android_res_base, f"mipmap-{name}")
        os.makedirs(folder, exist_ok=True)
        
        # Limpa pngs antigos se houver
        for png_name in ["ic_launcher.png", "ic_launcher_round.png"]:
            p = os.path.join(folder, png_name)
            if os.path.exists(p):
                os.remove(p)

        resized = img.resize((size, size), Image.Resampling.LANCZOS)
        resized.save(os.path.join(folder, "ic_launcher.webp"), format="WEBP")
        resized.save(os.path.join(folder, "ic_launcher_round.webp"), format="WEBP")
        print(f"Icone {name} ({size}x{size}) atualizado com sucesso em WEBP!")

    # 3. Atualiza strings.xml com o nome elegante do app
    strings_xml_path = os.path.join(android_res_base, "values", "strings.xml")
    if os.path.exists(strings_xml_path):
        with open(strings_xml_path, "w", encoding="utf-8") as f:
            f.write('<resources>\n    <string name="app_name">App Compras</string>\n</resources>\n')
        print("Nome do app atualizado para 'App Compras'!")

    # 4. Garante o Deep Link scheme 'appcompras' no AndroidManifest.xml
    manifest_path = os.path.join(ROOT_DIR, "target", "dx", "frontend", "release", "android", "app", "app", "src", "main", "AndroidManifest.xml")
    if os.path.exists(manifest_path):
        with open(manifest_path, "r", encoding="utf-8") as f:
            manifest_content = f.read()
        if 'android:scheme="appcompras"' not in manifest_content:
            target_tag = "</intent-filter>"
            deep_link_xml = '</intent-filter>\n            <intent-filter>\n                <action android:name="android.intent.action.VIEW" />\n                <category android:name="android.intent.category.DEFAULT" />\n                <category android:name="android.intent.category.BROWSABLE" />\n                <data android:scheme="appcompras" />\n            </intent-filter>'
            manifest_content = manifest_content.replace(target_tag, deep_link_xml, 1)
            with open(manifest_path, "w", encoding="utf-8") as f:
                f.write(manifest_content)
            print("Deep Link 'appcompras://' adicionado ao AndroidManifest!")

