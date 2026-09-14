  1. 재귀 탐색 추가
    - 현재 디렉토리뿐만 아니라 모든 하위 디렉토리의 PDF 파일도 검색합니다
    - *.pdf → **/*.pdf 패턴 사용
  2. 디렉토리 구조 유지
    - 원본 파일의 디렉토리 구조를 toc_extracted 폴더 내에 그대로 유지합니다
    - 예: subfolder/file.pdf → toc_extracted/subfolder/file_TOC.pdf
  3. 중복 처리 방지
    - toc_extracted 폴더 내의 파일은 자동으로 제외됩니다

  사용 방법은 동일합니다:
  extract_toc.bat

  이제 하위 디렉토리의 모든 PDF 파일에서 목차를 추출하고, 원본 폴더 구조를 유지하여 저장합니다!
