@echo off
chcp 65001 >nul
echo ========================================
echo PDF 목차 추출 스크립트
echo ========================================
echo.

REM pypdf 라이브러리 설치 확인 및 자동 설치
python -c "import pypdf" 2>nul
if errorlevel 1 (
    echo pypdf 라이브러리가 설치되어 있지 않습니다.
    echo 자동으로 설치를 시작합니다...
    echo.
    pip install pypdf
    if errorlevel 1 (
        echo.
        echo 설치 실패. 수동으로 설치해주세요: pip install pypdf
        pause
        exit /b 1
    )
    echo.
    echo 설치 완료!
    echo.
)

REM Python 스크립트 실행
python extract_toc.py

echo.
pause
