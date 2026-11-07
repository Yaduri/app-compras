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
android_app_base = os.path.join(
    ROOT_DIR, "target", "dx", "frontend", "release", "android", "app", "app", "src", "main"
)
android_res_base = os.path.join(android_app_base, "res")

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

    # 4. Garante o Deep Link scheme 'appcompras' e singleTask no AndroidManifest.xml
    manifest_path = os.path.join(android_app_base, "AndroidManifest.xml")
    if os.path.exists(manifest_path):
        with open(manifest_path, "r", encoding="utf-8") as f:
            manifest_content = f.read()
        
        # Adiciona launchMode singleTask se não estiver presente
        if 'android:launchMode="singleTask"' not in manifest_content:
            manifest_content = manifest_content.replace(
                '<activity android:configChanges="orientation|screenLayout|screenSize|keyboardHidden"',
                '<activity android:launchMode="singleTask" android:configChanges="orientation|screenLayout|screenSize|keyboardHidden"'
            )
            print("Configurado android:launchMode='singleTask' na MainActivity!")

        # Adiciona intent-filter do deep link appcompras://
        if 'android:scheme="appcompras"' not in manifest_content:
            target_tag = "</intent-filter>"
            deep_link_xml = '</intent-filter>\n            <intent-filter>\n                <action android:name="android.intent.action.VIEW" />\n                <category android:name="android.intent.category.DEFAULT" />\n                <category android:name="android.intent.category.BROWSABLE" />\n                <data android:scheme="appcompras" />\n            </intent-filter>'
            manifest_content = manifest_content.replace(target_tag, deep_link_xml, 1)
            print("Deep Link 'appcompras://' adicionado ao AndroidManifest!")

        with open(manifest_path, "w", encoding="utf-8") as f:
            f.write(manifest_content)

    # 5. Configura MainActivity.kt com captura de Deep Link e injeção no WebView
    main_activity_path = os.path.join(android_app_base, "kotlin", "dev", "dioxus", "main", "MainActivity.kt")
    if os.path.exists(main_activity_path):
        main_activity_code = """package dev.dioxus.main

import android.content.Intent
import android.os.Bundle
import android.webkit.WebView

typealias BuildConfig = com.appcompras.lista.BuildConfig

class MainActivity : WryActivity() {
    private var webViewInstance: WebView? = null
    private var pendingHash: String? = null

    override fun onWebViewCreate(webView: WebView) {
        super.onWebViewCreate(webView)
        webViewInstance = webView
        pendingHash?.let { hash ->
            injectHash(hash)
            pendingHash = null
        }
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        handleIntent(intent)
    }

    override fun onNewIntent(intent: Intent) {
        super.onNewIntent(intent)
        setIntent(intent)
        handleIntent(intent)
    }

    private fun handleIntent(intent: Intent?) {
        val data = intent?.data ?: return
        if (data.scheme == "appcompras") {
            val url = data.toString()
            val hash = if (url.contains("#")) {
                url.substringAfter("#")
            } else {
                data.fragment ?: ""
            }
            if (hash.isNotEmpty()) {
                if (webViewInstance != null) {
                    injectHash(hash)
                } else {
                    pendingHash = hash
                }
            }
        }
    }

    private fun injectHash(hash: String) {
        webViewInstance?.post {
            val js = "(function(){ window.location.hash = '$hash'; if(window.onAppComprasToken) { window.onAppComprasToken('$hash'); } else { window.dispatchEvent(new HashChangeEvent('hashchange')); } })();"
            webViewInstance?.evaluateJavascript(js, null)
        }
    }
}
"""
        with open(main_activity_path, "w", encoding="utf-8") as f:
            f.write(main_activity_code)
        print("MainActivity.kt configurada com suporte a Deep Links e OAuth!")

    # 6. Configura RustWebViewClient.kt para abrir OAuth no Chrome e tratar appcompras://
    webview_client_path = os.path.join(android_app_base, "kotlin", "dev", "dioxus", "main", "RustWebViewClient.kt")
    if os.path.exists(webview_client_path):
        with open(webview_client_path, "r", encoding="utf-8") as f:
            client_code = f.read()

        old_method = """    override fun shouldOverrideUrlLoading(
        view: WebView,
        request: WebResourceRequest
    ): Boolean {
        return shouldOverride(request.url.toString())
    }"""

        new_method = """    override fun shouldOverrideUrlLoading(
        view: WebView,
        request: WebResourceRequest
    ): Boolean {
        val uri = request.url
        val url = uri.toString()
        if (url.contains("supabase.co/auth") || url.contains("accounts.google.com")) {
            try {
                val intent = android.content.Intent(android.content.Intent.ACTION_VIEW, uri)
                intent.flags = android.content.Intent.FLAG_ACTIVITY_NEW_TASK
                view.context.startActivity(intent)
                return true
            } catch (e: Exception) {
                Logger.warn("Failed to open external browser: $e")
            }
        }
        if (url.startsWith("appcompras://")) {
            val hash = if (url.contains("#")) url.substringAfter("#") else uri.fragment ?: ""
            if (hash.isNotEmpty()) {
                val js = "(function(){ window.location.hash = '$hash'; if(window.onAppComprasToken) window.onAppComprasToken('$hash'); else window.dispatchEvent(new HashChangeEvent('hashchange')); })();"
                view.evaluateJavascript(js, null)
            }
            return true
        }
        return shouldOverride(request.url.toString())
    }"""

        if old_method in client_code:
            client_code = client_code.replace(old_method, new_method)
            with open(webview_client_path, "w", encoding="utf-8") as f:
                f.write(client_code)
            print("RustWebViewClient.kt configurado com abertura de navegador externo!")
