@echo off
echo ====================================================
echo  Compilando App Compras para Android (APK)
echo ====================================================

set ANDROID_HOME=%LOCALAPPDATA%\Android\Sdk
set NDK_HOME=%LOCALAPPDATA%\Android\Sdk\ndk\28.2.13676358

if exist "C:\Program Files\Android\Android Studio\jbr" (
    set "JAVA_HOME=C:\Program Files\Android\Android Studio\jbr"
)
set "PATH=%JAVA_HOME%\bin;%PATH%"

echo [1/3] Variaveis configuradas:
echo       ANDROID_HOME=%ANDROID_HOME%
echo       NDK_HOME=%NDK_HOME%
echo       JAVA_HOME=%JAVA_HOME%
echo.
echo [2/3] Sincronizando icones Android em alta resolucao...
python scripts\setup_android_icons.py

echo.
echo [3/3] Iniciando compilacao via Dioxus CLI para ARM64 (Celulares Fisicos)...

cd frontend
dx build --release --platform android --target aarch64-linux-android
cd ..

if not exist "dist" mkdir "dist"
if exist "target\dx\frontend\release\android\app\app\build\outputs\apk\debug\app-debug.apk" (
    copy /Y "target\dx\frontend\release\android\app\app\build\outputs\apk\debug\app-debug.apk" "dist\app-compras.apk" >nul
)

echo.
echo ====================================================
echo  APK gerado com sucesso!
echo  Arquivo: dist\app-compras.apk
echo ====================================================
pause
