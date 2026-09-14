#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
PDF 목차(Table of Contents) 페이지 추출 스크립트
"""

import os
import sys
from pathlib import Path

try:
    from pypdf import PdfReader, PdfWriter
except ImportError:
    print("pypdf 라이브러리가 설치되어 있지 않습니다.")
    print("설치 명령: pip install pypdf")
    sys.exit(1)


def has_toc_keywords(text):
    """텍스트에 목차 관련 키워드가 있는지 확인"""
    if not text:
        return False

    text_lower = text.lower()
    keywords = [
        'table of contents',
        'contents',
        '목차',
        'index',
        'table des matières',  # 프랑스어
        'inhaltsverzeichnis',  # 독일어
    ]

    # 키워드가 페이지 시작 부분(처음 500자 이내)에 있는지 확인
    text_start = text_lower[:500]
    for keyword in keywords:
        if keyword in text_start:
            return True
    return False


def find_toc_pages(pdf_path):
    """PDF에서 목차 페이지 번호들을 찾기"""
    try:
        reader = PdfReader(pdf_path)
        toc_pages = []

        for page_num, page in enumerate(reader.pages):
            try:
                text = page.extract_text()
                if has_toc_keywords(text):
                    toc_pages.append(page_num)
                    print(f"  - 목차 페이지 발견: {page_num + 1}페이지")
            except Exception as e:
                print(f"  - {page_num + 1}페이지 읽기 오류: {e}")
                continue

        return toc_pages
    except Exception as e:
        print(f"  - PDF 읽기 오류: {e}")
        return []


def extract_toc_pages(pdf_path, output_dir="toc_extracted", base_dir="."):
    """목차 페이지를 추출하여 새 PDF로 저장"""
    pdf_path = Path(pdf_path)
    base_dir = Path(base_dir)

    # 원본 파일의 상대 경로 구조 유지
    try:
        relative_path = pdf_path.relative_to(base_dir)
    except ValueError:
        relative_path = pdf_path

    print(f"\n처리중: {relative_path}")

    toc_pages = find_toc_pages(str(pdf_path))

    if not toc_pages:
        print(f"  → 목차 페이지를 찾을 수 없습니다. 스킵합니다.")
        return False

    # 출력 디렉토리 구조 생성 (원본 디렉토리 구조 유지)
    output_subdir = Path(output_dir) / relative_path.parent
    os.makedirs(output_subdir, exist_ok=True)

    # 목차 페이지만 추출
    try:
        reader = PdfReader(str(pdf_path))
        writer = PdfWriter()

        for page_num in toc_pages:
            writer.add_page(reader.pages[page_num])

        output_filename = f"{pdf_path.stem}_TOC.pdf"
        output_path = output_subdir / output_filename

        with open(output_path, 'wb') as output_file:
            writer.write(output_file)

        print(f"  → 추출 완료: {output_path} ({len(toc_pages)}페이지)")
        return True
    except Exception as e:
        print(f"  → 추출 실패: {e}")
        return False


def main():
    """메인 함수"""
    # 현재 디렉토리 및 하위 디렉토리의 모든 PDF 파일 찾기 (재귀 탐색)
    current_dir = Path('.')
    pdf_files = list(current_dir.glob('**/*.pdf'))

    if not pdf_files:
        print("현재 디렉토리 및 하위 디렉토리에 PDF 파일이 없습니다.")
        return

    print(f"총 {len(pdf_files)}개의 PDF 파일을 찾았습니다. (하위 디렉토리 포함)\n")
    print("=" * 60)

    success_count = 0
    skip_count = 0

    for pdf_file in pdf_files:
        # toc_extracted 폴더 내의 파일은 제외
        if 'toc_extracted' in pdf_file.parts:
            continue

        if extract_toc_pages(pdf_file, output_dir="toc_extracted", base_dir=current_dir):
            success_count += 1
        else:
            skip_count += 1

    print("\n" + "=" * 60)
    print(f"\n작업 완료!")
    print(f"  - 성공: {success_count}개")
    print(f"  - 스킵: {skip_count}개")
    print(f"\n추출된 파일은 'toc_extracted' 폴더에 원본 디렉토리 구조를 유지하여 저장되었습니다.")


if __name__ == '__main__':
    main()
