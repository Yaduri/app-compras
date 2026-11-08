@echo off
setlocal EnableDelayedExpansion
echo ====================================================
echo  Compilando App Compras para Android (APK)
echo ====================================================

set ANDROID_HOME=%LOCALAPPDATA%\Android\Sdk
set NDK_HOME=%LOCALAPPDATA%\Android\Sdk\ndk\28.2.13676358

if exist "C:\Program Files\Android\Android Studio\jbr" (
    set "JAVA_HOME=C:\Program Files\Android\Android Studio\jbr"
)
set "PATH=%JAVA_HOME%\bin;%PATH%"

REM ----------------------------------------------------------------------
REM  Assinatura do APK de release. Defina estas variaveis antes de rodar
REM  (ou crie um arquivo signing.bat ao lado deste, fora do git, com os SET):
REM    set APP_KEYSTORE=C:\caminho\app-compras.jks
REM    set APP_KEYSTORE_PASS=senha-do-keystore
REM    set APP_KEY_ALIAS=appcompras
REM  Para criar o keystore uma unica vez (guarde-o com seguranca!):
REM    keytool -genkeypair -v -keystore app-compras.jks -alias appcompras -keyalg RSA -keysize 2048 -validity 10000
REM ----------------------------------------------------------------------
if exist "%~dp0signing.bat" call "%~dp0signing.bat"

set BUILD_MODE=release
if "%APP_KEYSTORE%"=="" set BUILD_MODE=debug
if not exist "%APP_KEYSTORE%" set BUILD_MODE=debug

echo [1/4] Variaveis configuradas:
echo       ANDROID_HOME=%ANDROID_HOME%
echo       NDK_HOME=%NDK_HOME%
echo       JAVA_HOME=%JAVA_HOME%
echo       Modo do APK=%BUILD_MODE%
echo.

echo [2/4] Compilando codigo Rust nativo para ARM64...
cd frontend
dx build --release --platform android --target aarch64-linux-android --no-default-features --features mobile
if errorlevel 1 (
    echo ERRO: falha ao compilar o codigo Rust.
    cd ..
    pause
    exit /b 1
)
cd ..

echo.
echo [3/4] Aplicando nome, icones e deep link seguro...
python scripts\setup_android_icons.py
if errorlevel 1 (
    echo ERRO: falha ao configurar o projeto Android.
    pause
    exit /b 1
)

echo.
echo [4/4] Montando APK final via Gradle...
cd target\dx\frontend\release\android\app
if not exist "local.properties" (
    echo sdk.dir=%ANDROID_HOME:\=\\% > local.properties
)

if "%BUILD_MODE%"=="release" (
    call gradlew.bat assembleRelease
) else (
    call gradlew.bat assembleDebug
)
if errorlevel 1 (
    echo ERRO: falha no Gradle.
    cd /d "%~dp0"
    pause
    exit /b 1
)
cd /d "%~dp0"

if not exist "dist" mkdir "dist"
set OUT_DIR=target\dx\frontend\release\android\app\app\build\outputs\apk

if "%BUILD_MODE%"=="release" (
    REM Localiza a versao mais recente do build-tools (zipalign e apksigner)
    set BUILD_TOOLS=
    for /f "delims=" %%d in ('dir /b /ad /o:n "%ANDROID_HOME%\build-tools"') do set "BUILD_TOOLS=%ANDROID_HOME%\build-tools\%%d"
    if "!BUILD_TOOLS!"=="" (
        echo ERRO: Android build-tools nao encontrado em %ANDROID_HOME%\build-tools
        pause
        exit /b 1
    )

    set UNSIGNED=%OUT_DIR%\release\app-release-unsigned.apk
    if not exist "!UNSIGNED!" set UNSIGNED=%OUT_DIR%\release\app-release.apk

    "!BUILD_TOOLS!\zipalign.exe" -f -p 4 "!UNSIGNED!" "dist\app-compras-aligned.apk"
    if errorlevel 1 (
        echo ERRO: zipalign falhou.
        pause
        exit /b 1
    )
    call "!BUILD_TOOLS!\apksigner.bat" sign --ks "%APP_KEYSTORE%" --ks-pass env:APP_KEYSTORE_PASS --ks-key-alias "%APP_KEY_ALIAS%" --out "dist\app-compras.apk" "dist\app-compras-aligned.apk"
    if errorlevel 1 (
        echo ERRO: assinatura do APK falhou.
        pause
        exit /b 1
    )
    del "dist\app-compras-aligned.apk" >nul 2>&1
    echo.
    echo  APK de RELEASE assinado gerado: dist\app-compras.apk
) else (
    if exist "%OUT_DIR%\debug\app-debug.apk" (
        copy /Y "%OUT_DIR%\debug\app-debug.apk" "dist\app-compras.apk" >nul
    )
    echo.
    echo  ATENCAO: APK de DEBUG gerado ^(apenas para testes^).
    echo  Ele e depuravel e nao pode ser publicado na Play Store.
    echo  Configure APP_KEYSTORE / APP_KEYSTORE_PASS / APP_KEY_ALIAS para gerar o release.
)

echo.
echo ====================================================
echo  Arquivo: dist\app-compras.apk
echo ====================================================
pause
