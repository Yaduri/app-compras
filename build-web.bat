@echo off
echo ====================================================
echo  Compilando App Compras para Web / PWA Mobile
echo ====================================================

cd frontend
dx build --release --platform web

echo.
echo Concluido! Os arquivos Web/PWA estarao em frontend/dist/.
pause
