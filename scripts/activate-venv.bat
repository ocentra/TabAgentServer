@echo off
REM Activate Python 3.11 venv with all dependencies
call venv-py311\Scripts\activate.bat
echo ✅ Python 3.11 venv activated
echo.
echo Available backends:
python cli.py --format json backends
echo.
echo Ready to use!

