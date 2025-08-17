@echo off
echo Building md-to-pdf converter...
cargo build --release
if %ERRORLEVEL% NEQ 0 (
    echo Build failed!
    pause
    exit /b 1
)

echo Build successful! Testing conversion...
target\release\md-to-pdf.exe -i test.md -o test.pdf
if %ERRORLEVEL% NEQ 0 (
    echo Conversion failed!
    pause
    exit /b 1
)

echo Conversion successful! Check test.pdf
pause