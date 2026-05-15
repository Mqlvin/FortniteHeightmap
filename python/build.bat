REM this batch thing is so wacky

@echo off
if exist export_server.exe del /f /q export-server.exe

pyinstaller --onefile --name export-server .\export_server.py
if errorlevel 1 (
  echo PyInstaller failed
  exit /b 1
)

if exist export-server.spec del /f /q export-server.spec

if exist .\dist\export-server.exe copy /y .\dist\export-server.exe .\

if exist dist rmdir /s /q dist
if exist build rmdir /s /q build
