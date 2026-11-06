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

echo [1/4] Variaveis configuradas:
echo       ANDROID_HOME=%ANDROID_HOME%
echo       NDK_HOME=%NDK_HOME%
echo       JAVA_HOME=%JAVA_HOME%
echo.

echo [2/4] Compilando codigo Rust nativo para ARM64...
cd frontend
dx build --release --platform android --target aarch64-linux-android --no-default-features --features mobile
cd ..

echo.
echo [3/4] Aplicando nome 'App Compras' e icones minimalistas...
python scripts\setup_android_icons.py

echo.
echo [4/4] Montando APK final via Gradle...
cd target\dx\frontend\release\android\app
if not exist "local.properties" (
    echo sdk.dir=%ANDROID_HOME:\=\\% > local.properties
)
call gradlew.bat assembleDebug
cd /d "%~dp0"

if not exist "dist" mkdir "dist"
if exist "target\dx\frontend\release\android\app\app\build\outputs\apk\debug\app-debug.apk" (
    copy /Y "target\dx\frontend\release\android\app\app\build\outputs\apk\debug\app-debug.apk" "dist\app-compras.apk" >nul
)

echo.
echo ====================================================
echo  APK gerado com sucesso!
echo  Nome: App Compras
echo  Arquivo: dist\app-compras.apk
echo ====================================================
pause
