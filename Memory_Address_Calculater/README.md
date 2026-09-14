# 메모리 주소 변환 계산기

T32 디버거 사용 시 ROM 주소를 RAM 주소로 변환하는 도구입니다.

## 동작 원리

```
변환된 주소 = Working Base + (입력 주소 - Reference Base)
```

**예시:**
```
입력 주소     : 0xA021DCBC  (ROM)
Reference Base: 0xA021C000
Working Base  : 0x60100000
오프셋        : 0x00001CBC
변환된 주소   : 0x60101CBC  (RAM)
```

## 빌드 및 실행

```bash
# 빌드
cargo build --release

# 실행
./target/release/memory_address_calculator
```

## 설정

`config.txt` 파일을 수정하여 베이스 주소를 설정합니다:

```txt
reference_base=0xA021C000  # ROM 시작 주소
working_base=0x60100000    # RAM 시작 주소
```

## 사용법

```bash
# 주소 입력 (0x 접두사는 선택사항)
Enter address > A021DCBC

# 결과 확인 (자동으로 클립보드에 복사됨)
Converted Address   : 0x60101CBC
RESULT=0x60101CBC
Clipboard Copy      : Success

# 종료
Enter address > q
```

## 주의사항

### ✅ 정상 동작 조건

이 도구는 **선형 1:1 매핑**을 가정합니다:
- ROM과 RAM의 배치 순서가 동일해야 함
- 섹션 간 오프셋이 일정해야 함
- 동일한 빌드 환경 (컴파일러, 링커 스크립트)

### ⚠️ 사용 전 확인 사항

**1. 링커 맵 파일 확인**
```bash
# LMA(ROM)와 VMA(RAM)의 오프셋이 일정한지 확인
cat firmware.map | grep ".data"
```

**2. ELF 섹션 정보 확인**
```bash
arm-none-eabi-objdump -h firmware.elf
# LMA와 VMA 컬럼 비교
```

**3. 링커 스크립트 검증**
```ld
SECTIONS {
    .data : {
        *(.data)
    } > RAM AT > ROM  /* 1:1 매핑 확인 */
}
```

### ❌ 동작하지 않는 경우

- 여러 섹션이 서로 다른 순서로 배치된 경우
- ROM과 RAM의 정렬(alignment)이 다른 경우
- 뱅크 스위칭 등 복잡한 메모리 구조

## 다른 프로젝트에 적용

`config.txt`만 수정하면 다른 프로젝트에도 사용 가능합니다:

```txt
# STM32 예시
reference_base=0x08000000  # Flash
working_base=0x20000000    # SRAM

# nRF52 예시
reference_base=0x00000000  # Flash
working_base=0x20000000    # RAM
```

## 제한사항

- ROM → RAM 단방향 변환만 지원
- 한 번에 하나의 주소만 처리
- 선형 매핑만 지원 (복잡한 메모리 레이아웃 불가)

## 테스트

```bash
cargo test
```

## 문제 해결

**"Input address is below reference base" 오류**
- 입력 주소가 reference base보다 작음
- config.txt의 reference_base 값 확인

**클립보드 복사 실패 (Linux)**
```bash
sudo apt-get install xclip
```

**변환 결과가 부정확함**
- 링커 스크립트의 메모리 매핑 확인
- 맵 파일의 실제 심볼 주소와 비교
