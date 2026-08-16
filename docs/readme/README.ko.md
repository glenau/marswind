<div align="center">

<img src="../../src-tauri/icons/128x128@2x.png" alt="" width="128" height="128">

# Marswind

**컴퓨터에서 재생되는 소리를 위한 실시간 자막과 번역. 완전 오프라인.**

[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](../../LICENSE)
[![Platform: macOS 14.4+](https://img.shields.io/badge/platform-macOS%2014.4%2B-lightgrey.svg)](#지원-플랫폼)
[![Version](https://img.shields.io/badge/version-0.1.1-brightgreen.svg)](#)

[English](../../README.md) ·
[Русский](README.ru.md) ·
[Deutsch](README.de.md) ·
[Español](README.es.md) ·
[Français](README.fr.md) ·
[Italiano](README.it.md) ·
[Português](README.pt.md) ·
[Polski](README.pl.md) ·
[Türkçe](README.tr.md) ·
[Українська](README.uk.md) ·
[中文](README.zh.md) ·
[日本語](README.ja.md) ·
**한국어**

<img src="../screenshot.png" alt="Marswind 창: 왼쪽은 원문, 오른쪽은 스페인어 번역" width="900">

</div>

Marswind는 컴퓨터에서 재생 중인 소리 - YouTube 영상, Google Meet·Teams·Zoom 통화,
로컬 비디오 파일 - 를 듣고 음성을 인식해, 말하는 도중에 원하는 언어로 번역합니다.

API 키도, 계정도, 인터넷도 필요 없습니다. 모델은 한 번만 내려받아 이후에는 로컬에서
실행되며, 오디오는 메모리에만 머물러 디스크에 기록되거나 외부로 전송되지 않습니다.

## 무엇을 하나

- **시스템 오디오를 캡처**합니다. 가상 오디오 드라이버가 필요 없으며, 컴퓨터가 내는
  모든 소리 또는 브라우저 같은 단일 앱만 골라서 들을 수 있습니다
- **음성을 인식**합니다. GPU에서 whisper.cpp로 동작하며, 자막은 읽는 사람 눈앞에서
  고쳐 쓰이는 대신 말이 이어지는 대로 늘어납니다
- **말하는 도중에 번역**합니다. 단어는 문장이 끝나기를 기다리지 않고 확정되는 즉시
  번역기로 넘어가며, 번역문도 한 단어씩 도착합니다
- **모델을 앱 안에서 관리**합니다. 인식 모델 6종과 번역 모델 3종을 진행률 표시와
  SHA-256 검증과 함께 내려받으며, 모두 MIT 또는 Apache-2.0입니다
- **모든 세션을 기록**합니다. 나중에 살펴볼 수 있고 텍스트, 자막(`.srt`), 또는 타이밍
  정보가 담긴 JSON으로 내보낼 수 있습니다
- **예제 음성 클립을 포함**하고 있어 영상을 찾지 않고도 시험해 볼 수 있습니다
- **13개 언어를 지원**합니다. 번역 대상과 같은 언어들이며, 라이트/다크 테마와 글자만이
  아니라 인터페이스 전체를 확대·축소하는 글자 크기 설정을 제공합니다

### 언어

영어, 러시아어, 독일어, 스페인어, 프랑스어, 이탈리아어, 포르투갈어, 폴란드어,
터키어, 우크라이나어, 중국어, 일본어, 한국어 - 번역 대상 언어로도, 창 자체의 언어로도
쓸 수 있습니다. 말하는 언어는 기본적으로 오디오에서 자동 판별하며, 인식 범위는
whisper가 지원하는 전부입니다.

## 작동 방식

```
시스템 오디오  →  16 kHz 모노로 리샘플링  →  음성 구간 검출(Silero)
              →  음성 인식(whisper.cpp)
              →  번역(llama.cpp, 별도 프로세스)
              →  전사: 왼쪽에 원문, 그 옆에 번역
```

인터페이스 아래쪽은 모두 Rust로 작성되어 전용 스레드에서 돌아가며, 번역은 별도의
실행 파일에 있습니다. whisper.cpp와 llama.cpp가 한 프로세스를 공유할 수 없기
때문입니다. 설계와 그 근거는 [docs/ARCHITECTURE.md](../ARCHITECTURE.md)에 있습니다.

Apple Silicon에서 기본 모델로, [tests/](../../tests/README.md)의 합성 음성 코퍼스를
대상으로 측정했습니다 - 클립마다 세 번 실행한 중앙값입니다. 첫 자막은 클립 시작 후
약 6초, 이후로는 2~3초마다 새 자막이 나오며, 단어 오류율은 또렷한 낭독에서 4%, 고유
명사와 숫자가 이어지는 클립에서 23%였습니다. 인식은 결정적이지 않고 한 번의 실행이
스무 포인트가량 흔들리므로 이것은 결과가 아니라 중앙값입니다. 숫자를 얻는 방법은 테스트
장치 옆에 적어 두었습니다.

## 지원 플랫폼

| 플랫폼 | 상태 |
|---|---|
| **macOS 14.4+** | 지원 - Core Audio process taps, Metal |
| **Windows** | 개발 중 - WASAPI loopback |
| **Linux** | 개발 중 - PipeWire |

앱은 지금도 Windows와 Linux에서 빌드되고 실행되지만, 그곳에서는 오디오 캡처가 사용
불가로 보고됩니다. 즉 들을 것이 없는 창입니다. 캡처 위의 모든 계층은 이미 플랫폼과
무관하게 동작합니다.

BlackHole 같은 가상 오디오 드라이버는 **어느 플랫폼에서도 필요하지 않습니다**. 캡처는
운영체제의 네이티브 API를 통합니다.

## 요구 사항

| | |
|---|---|
| macOS | 14.4 이상, Apple Silicon 또는 Intel |
| 메모리 | 인식만 8 GB, 번역까지 16 GB |
| 디스크 | 선택한 모델에 따라 0.1-6.5 GB |
| 빌드에 필요 | [Rust](https://rustup.rs), [Node.js](https://nodejs.org) 20+, cmake (`brew install cmake`) |

## 설치

### 내려받기

[최신 릴리스](https://github.com/glenau/marswind/releases/latest)에 `.dmg`가
있습니다. 열어서 Marswind를 응용 프로그램으로 끌어다 놓으면 끝이며, 약 13 MB입니다.
모델은 나중에, 고른 것만 내려받습니다.

**macOS는 처음에는 열기를 거부합니다.** 이미지에는 서명이 있지만 공증은 받지
않았습니다. 이 프로젝트에는 유료 Developer ID 인증서가 없고, Gatekeeper는 인증서가
없는 것을 모두 신원 미상으로 취급합니다. 통과하는 방법:

1. 앱을 열어 차단되도록 둡니다. **Done**을 누르세요. "Move to Bin"이 아닙니다.
2. **시스템 설정 → 개인정보 보호 및 보안**에서 아래 **보안** 항목까지 스크롤합니다.
   Marswind가 차단되었다는 줄과 그 옆에 **그래도 열기** 버튼이 있습니다.
3. 그것을 누르고 Touch ID나 암호로 인증한 뒤 한 번 더 확인합니다.

macOS는 한 번만 묻고 기억합니다. 이 버튼은 차단된 실행 뒤에만 나타나고 한 시간쯤
유지됩니다. 보이지 않으면 앱을 다시 열어 보세요.

앱을 오른쪽 클릭해 열기를 고르는 것은 예전 지름길이고 macOS 14에서는 아직
동작합니다. macOS 15에서 없어졌으므로 어디서나 통하는 길은 설정을 거치는 쪽입니다.

### 또는 직접 빌드하기

```bash
git clone https://github.com/glenau/marswind.git
cd marswind
npm install
npm run install:macos
```

번역 워커와 릴리스 번들을 빌드하고 ad-hoc 서명한 뒤 `/Applications/Marswind.app`으로
복사합니다. 첫 빌드는 몇 분 걸립니다 - whisper.cpp와 llama.cpp를 소스에서 컴파일하기
때문입니다.그 밖에 필요한 것은 없습니다. 받아 올 서브모듈도, 손으로 설치할
라이브러리도, 미리 내려받을 모델도 없습니다.

### 첫 실행

1. `open /Applications/Marswind.app`
2. macOS가 **오디오 녹음** 권한을 요청합니다. 허용하세요 - 없으면 앱이 아무것도 듣지
   못합니다. 거부했다면 시스템 설정 → 개인정보 보호 및 보안 → 오디오 녹음에서 다시
   허용할 수 있습니다.
3. **설정**을 열고 인식 모델과 번역 모델을 하나씩 내려받습니다. 16 GB 이상이면
   `Large v3 Turbo (compressed)`와 `Qwen3 4B Instruct`가 기본값이고, `Small`과
   `Qwen3 1.7B`는 8 GB에 들어갑니다. 합쳐서 약 3 GB이며, 모두 공개된 체크섬과
   대조해 확인합니다. 각 줄에는 해당 가중치의 라이선스가 표시됩니다 -
   [docs/MODELS.md](../MODELS.md)를 보세요.
4. **듣기 시작**을 누르고 말소리가 있는 것을 재생하세요. 영상을 찾기 번거롭다면
   설정에 샘플 클립 네 개가 들어 있습니다.

직접 빌드한 사본에 대해 알아둘 두 가지:

- **ad-hoc 서명입니다.** 같은 빌드에서는 서명이 유지되므로 오디오 권한도 남아 있지만,
  다시 빌드하면 새로운 identity가 되어 macOS가 다시 묻습니다. 이를 끝내는 것은
  Developer ID 인증서이며 아직 없습니다.
- **실행 중에는 앱을 옮기지 마세요.** 업데이트하려면 `npm run install:macos`를 다시
  실행하면 됩니다. `/Applications/Marswind.app`을 제자리에서 교체합니다.

### 업데이트

**설정 → 정보 → 업데이트 확인.** 더 새로운 릴리스가 있는지 GitHub에 묻고, 있으면
이미지를 「다운로드」에 내려받아 옆에 공개된 체크섬과 대조한 뒤 Finder에서 보여 줍니다.
설치는 처음과 같은 끌어다 놓기입니다.

스스로 확인하는 일은 없습니다. 타이머도, 실행 시 확인도 없습니다. 버튼을 누르지 않은
네트워크 요청은 하지 않기 때문입니다.

직접 빌드한 복사본은 설치한 방식 그대로 갱신합니다. `npm run install:macos`를 다시
실행하세요.

### 디스크 이미지 만들기

```bash
npm run build:dmg
```

릴리스 번들을 빌드하고 서명한 뒤
`src-tauri/target/Marswind-<버전>-<아키텍처>.dmg`로 묶습니다 - 릴리스에 첨부되는
것과 같은 이미지이며, 위와 같은 Gatekeeper 주의사항이 그대로 적용됩니다. 그 주변
절차는 [docs/RELEASING.md](../RELEASING.md)에 있습니다.

## 개발

`tauri dev`는 `Info.plist`도 서명도 없는 맨 실행 파일을 만들고, 그 형태에서는 Core
Audio process tap이 동작하지 않습니다. 대신 디버그 번들을 빌드·서명·실행하는 다음
명령을 쓰세요:

```bash
npm run dev:macos
```

| 명령 | 하는 일 |
|---|---|
| `npm run dev:macos` | 디버그 번들 빌드·서명·실행 |
| `npm run install:macos` | 릴리스 번들 빌드 및 설치 |
| `npm run check` | Svelte 및 TypeScript 타입 검사 |
| `npm run build:dmg` | 남에게 건넬 서명된 `.dmg` |
| `npm run build:sidecar` | 번역 워커만 빌드 |
| `npm run build:icons` | `scripts/make-icon.py`로 아이콘 다시 그리기 |
| `npm run build:social` | GitHub가 링크에 보여 주는 미리보기 카드를 다시 그리기 |
| `npm run licenses` | 잠금 파일에서 `THIRD-PARTY-NOTICES.md` 다시 생성 |

CI는 없습니다. whisper.cpp와 llama.cpp를 소스에서 컴파일하고, 테스트 도구가 시스템
출력으로 오디오를 재생하기 때문에 모든 검사는 로컬 명령입니다. 목록은
[CONTRIBUTING.md](../../CONTRIBUTING.md)에 있습니다.

## 테스트

유닛 테스트는 순수 로직을 다룹니다. [tests/](../../tests/README.md)의 스크립트는 시스템
출력으로 오디오를 재생하고 실제 파이프라인에서 나온 결과를 채점합니다 - 인식, 번역,
지연을 함께 봅니다.

```bash
npm run build:sidecar
cargo test --manifest-path src-tauri/Cargo.toml
```

첫 줄은 한 번만 필요하고, 이후로는 `cargo clean` 뒤에만 필요합니다. Tauri는 번역
워커를 사이드카로 묶기 때문에, 그 바이너리가 없으면 빌드 스크립트가 `src-tauri` 빌드
자체를 거부합니다. 새로 클론한 상태에서 `cargo test`만 실행하면
`resource path 'binaries/marswind-translator-…' doesn't exist`에서 멈춥니다.
`npm run` 빌드 명령은 이 단계를 대신 해 주지만, `cargo`를 직접 부르면 그렇지 않습니다.

파이프라인 스크립트에는 빌드하고 서명한 번들과 설치된 모델이 필요합니다:

```bash
npm run dev:macos
tests/run-capture.sh
tests/run-asr.sh
tests/run-pipeline.sh
```

이 코퍼스에서 한 번의 실행은 단어 오류율이 20포인트 가까이 흔들리므로, 숫자 하나만으로는
아무 의미가 없습니다. 여러 실행의 중앙값을 비교하고 점수뿐 아니라 전사 결과도 읽으세요.

## 개인정보

- 오디오는 **메모리에서** 캡처·리샘플링·인식됩니다. 디스크에 기록되거나 어딘가로
  전송되는 일은 없습니다.
- 네트워크 트래픽은 버튼을 눌렀을 때만 생깁니다. 모델 내려받기와 업데이트 확인,
  두 가지뿐이며 타이머나 실행 시 통신은 없습니다.
- 원격 측정, 분석, 크래시 리포트, 계정 모두 없습니다.
- 전사 기록은 기록 보기에 보여줄 내용을 위해 앱 데이터 디렉터리에만 저장됩니다. 앱
  안에서 삭제할 수 있습니다.

## 기여

버그 리포트, 아이디어, 풀 리퀘스트를 환영합니다.
[CONTRIBUTING.md](../../CONTRIBUTING.md)에 설정, 검사 항목, 커밋 규칙, 리뷰가 보는 점이
정리되어 있습니다. 큰 작업을 시작하기 전에는 이슈를 먼저 열어 주세요 - 명백해 보이는
개선 몇 가지는 이미 시도했다가 측정 결과와 함께 되돌렸습니다.

- [행동 강령](../../CODE_OF_CONDUCT.md)
- [보안 정책](../../SECURITY.md) - 취약점은 이슈가 아니라 비공개로 신고해 주세요

## 사용 기술

| | | |
|---|---|---|
| [whisper.cpp](https://github.com/ggml-org/whisper.cpp) | MIT | 음성 인식, 그리고 함께 들어 있는 Silero VAD 구현 |
| [llama.cpp](https://github.com/ggml-org/llama.cpp) | MIT | 번역, 별도 프로세스에서 |
| [ggml](https://github.com/ggml-org/ggml) | MIT | 둘 아래에 있는 텐서 라이브러리와 Metal 백엔드 |
| [whisper-rs](https://codeberg.org/tazz4843/whisper-rs) | Unlicense | whisper.cpp의 Rust 바인딩 |
| [llama-cpp-2](https://github.com/utilityai/llama-cpp-rs) | MIT / Apache-2.0 | llama.cpp의 Rust 바인딩 |
| [Silero VAD](https://github.com/snakers4/silero-vad) | MIT | 구절 경계를 찾는 모델 |
| [Tauri](https://tauri.app) | MIT / Apache-2.0 | 창과 프로세스 경계 |
| [Svelte](https://svelte.dev) | MIT | 인터페이스 |
| [rubato](https://github.com/HEnquist/rubato) | MIT | whisper 앞단의 FFT 리샘플러 |

의존성 전체와 각 패키지의 라이선스는
[THIRD-PARTY-NOTICES.md](../../THIRD-PARTY-NOTICES.md)에 있습니다 - 잠금 파일에서
생성되며, 라이선스 자체와 나란히 앱 안에 함께 담깁니다.

**모델은 여기에 포함되지 않습니다.** 모델은 사용자의 요청에 따라
[Hugging Face](https://huggingface.co)에서 내려받으며 게시자의 조건을 그대로
유지합니다. 목록에는 읽지 않고 받아들일 수 있는 조건의 모델만 올립니다. whisper와
Silero 모델은 MIT, Qwen3는 Apache-2.0입니다. 설정의 각 줄은 내려받기가 시작되기
전에 라이선스를 알려 줍니다. 자세한 내용은 [docs/MODELS.md](../MODELS.md)에 있습니다.

## 라이선스

MIT - [LICENSE](../../LICENSE)를 보세요. 이는 이 저장소에 대한 것이며 모델에는
적용되지 않고, 위의 고지는 그것이 가리키는 라이선스를 대신하지 않습니다.
