<div align="center">

<img src="../../src-tauri/icons/128x128@2x.png" alt="" width="128" height="128">

# Marswind

**コンピューターの音声にリアルタイム字幕と翻訳を。完全オフライン。**

[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](../../LICENSE)
[![Platform: macOS 14.4+](https://img.shields.io/badge/platform-macOS%2014.4%2B-lightgrey.svg)](#対応プラットフォーム)
[![Version](https://img.shields.io/badge/version-0.1.0-brightgreen.svg)](#)

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
**日本語** ·
[한국어](README.ko.md)

<img src="../screenshot.png" alt="Marswind のウィンドウ：左が原文、右がスペイン語訳" width="900">

</div>

Marswind はマシンで再生されている音声-YouTube の動画、Google Meet や Teams、Zoom
の通話、ローカルの動画ファイル-を聞き取り、音声を認識して、話している最中に選んだ
言語へ翻訳します。

API キーもアカウントもインターネットも不要です。モデルは一度だけダウンロードされ、
その後はローカルで動きます。音声はメモリ上にとどまり、ディスクに書き出されることも、
どこかへ送られることもありません。

## できること

- **システム音声を取り込む**（仮想オーディオドライバ不要）-マシンが鳴らしている
  すべて、あるいはブラウザなど単一のアプリケーションだけ
- **音声を認識する**（GPU 上の whisper.cpp）：字幕は読み手の目の前で書き換わるので
  はなく、話されるにつれて伸びていきます
- **話しながら翻訳する**-単語は文の終わりを待たず、確定した時点で翻訳へ送られ、訳
  文も一語ずつ届きます
- **モデルをアプリ内で管理**：認識モデル 7 種と翻訳モデル 5 種を、進捗表示と
  SHA-256 検証つきでダウンロード
- **すべてのセッションを記録**-後から閲覧でき、テキスト・字幕（`.srt`）・タイミング
  付き JSON として書き出せます
- **サンプル音声を同梱**しているので、動画を探さなくても試せます
- **13 言語のインターフェース**-翻訳先と同じ言語-ライト／ダークテーマ、および文字
  だけでなく UI 全体を拡大縮小する文字サイズ設定

### 言語

英語・ロシア語・ドイツ語・スペイン語・フランス語・イタリア語・ポルトガル語・
ポーランド語・トルコ語・ウクライナ語・中国語・日本語・韓国語。翻訳先としても、
ウィンドウ自体の言語としても使えます。話されている言語は既定で音声から自動判定され、
認識は whisper が対応する範囲すべてをカバーします。

## しくみ

```
システム音声  →  16 kHz モノラルへリサンプリング  →  音声区間検出（Silero）
             →  音声認識（whisper.cpp）
             →  翻訳（llama.cpp、別プロセス）
             →  文字起こし：左に原文、その横に訳文
```

インターフェースより下はすべて Rust で書かれ、専用スレッドで動きます。翻訳が別バイナ
リなのは、whisper.cpp と llama.cpp が 1 つのプロセスを共有できないからです。設計と
その理由は [docs/ARCHITECTURE.md](../ARCHITECTURE.md) にあります。

Apple Silicon 上で既定のモデルを使い、[tests/](../../tests/README.md) の合成音声
コーパスで測定した値です（クリップごとに 3 回実行した中央値）。最初の字幕はクリップ
開始からおよそ 6 秒、以降は 2〜3 秒ごとに更新され、単語誤り率は明瞭な朗読で 4%、固有
名詞と数字が並ぶクリップで 23% でした。認識は決定的ではなく 1 回の実行で 20 ポイント
ほど動くため、これは結果ではなく中央値です。数値の出し方はテスト台のそばに書いてあり
ます。

## 対応プラットフォーム

| プラットフォーム | 状態 |
|---|---|
| **macOS 14.4+** | 対応済み - Core Audio process taps、Metal |
| **Windows** | 開発中 - WASAPI loopback |
| **Linux** | 開発中 - PipeWire |

アプリは現時点でも Windows と Linux でビルド・起動できますが、そこでは音声キャプチャ
が「利用不可」と報告します。つまり聞くものがないウィンドウです。キャプチャより上の層
はすでにプラットフォーム非依存で動作します。

BlackHole のような仮想オーディオドライバは**どのプラットフォームでも不要**です。
キャプチャは OS のネイティブ API を通ります。

## 動作要件

| | |
|---|---|
| macOS | 14.4 以降、Apple Silicon または Intel |
| メモリ | 認識のみで 8 GB、翻訳込みで 16 GB |
| ディスク | 選んだモデルに応じて 0.5〜4.5 GB |
| ビルドに必要 | [Rust](https://rustup.rs)、[Node.js](https://nodejs.org) 20 以降、cmake（`brew install cmake`） |

## インストール

### ダウンロードする

[最新リリース](https://github.com/glenau/marswind/releases/latest)に `.dmg` が
あります。開いて Marswind を Applications にドラッグするだけ、約 13 MB です。モデル
は後から、選んだものだけがダウンロードされます。

**macOS は初回の起動を拒否します。** イメージには署名がありますが公証は受けて
いません。このプロジェクトには有料の Developer ID 証明書がなく、Gatekeeper は
証明書のないものをすべて身元不明として扱います。通す手順は次のとおりです。

1. アプリを開いてブロックさせます。**Done** を押してください。「Move to Bin」
   ではありません。
2. **システム設定 → プライバシーとセキュリティ**を開き、下の**セキュリティ**まで
   スクロールします。Marswind がブロックされた旨の行と、その横に**このまま開く**
   ボタンがあります。
3. それを押し、Touch ID かパスワードで認証し、もう一度確認します。

macOS が尋ねるのは一度きりです。このボタンはブロックされた起動のあとにだけ現れ、
1 時間ほどで消えます。見当たらないときは、もう一度アプリを開いてください。

アプリを右クリックして「開く」を選ぶのは以前からの近道で、macOS 14 では今も
使えます。macOS 15 で廃止されたため、どの環境でも通るのは設定を経由する手順です。

### または自分でビルドする

```bash
git clone https://github.com/glenau/marswind.git
cd marswind
npm install
npm run install:macos
```

これで翻訳ワーカーとリリースバンドルがビルドされ、ad-hoc 署名のうえ
`/Applications/Marswind.app` にコピーされます。初回ビルドには数分かかります-
whisper.cpp と llama.cpp をソースからコンパイルするためです。ほかに必要なものはありません。取得すべきサブモジュールも、手で入れる
ライブラリも、先に落としておくモデルもありません。

### 初回起動

1. `open /Applications/Marswind.app`
2. macOS が**オーディオ録音**の許可を求めます。許可してください-なければアプリは何も
   聞こえません。拒否してしまった場合は、システム設定 → プライバシーとセキュリティ →
   オーディオ録音 から再度許可できます。
3. **設定**を開き、認識モデルと翻訳モデルを 1 つずつダウンロードします。16 GB
   以上のマシンでは `Large v3 Turbo (compressed)` と `Qwen3 4B Instruct` が既定で、
   `Small` と `Qwen3 1.7B` なら 8 GB に収まります。合計 3 GB ほどで、いずれも公開
   されたチェックサムと照合されます。各行にはその重みのライセンスが示されています -
   [docs/MODELS.md](../MODELS.md) を参照してください。
4. **聞き取りを開始**を押し、音声のあるものを再生します。動画を探すのが面倒なら、
   設定にサンプルクリップが 4 つ入っています。

自分でビルドしたコピーについて 2 点：

- **ad-hoc 署名です。** 同じビルドであれば署名は安定しているので録音許可は保持されま
  すが、リビルドすると別の identity になり macOS が再び尋ねます。これを解消するのは
  Developer ID 証明書で、まだ用意されていません。
- **実行中にアプリを移動しないでください。** 更新するには `npm run install:macos` を
  再実行します。`/Applications/Marswind.app` をその場で置き換えます。

### ディスクイメージの作成

```bash
npm run build:dmg
```

リリースバンドルをビルドし、署名し、
`src-tauri/target/Marswind-<バージョン>-<アーキテクチャ>.dmg` にまとめます。リリース
に添付されるのと同じイメージで、Gatekeeper についても上と同じ注意が当てはまります。
その周辺の手順は [docs/RELEASING.md](../RELEASING.md) にあります。

## 開発

`tauri dev` は `Info.plist` も署名もない素の実行ファイルを生成し、その形では Core
Audio の process tap は動きません。代わりに、デバッグバンドルをビルドして署名し起動
する次のコマンドを使ってください：

```bash
npm run dev:macos
```

| コマンド | 内容 |
|---|---|
| `npm run dev:macos` | デバッグバンドルをビルド・署名・起動 |
| `npm run install:macos` | リリースバンドルをビルドしてインストール |
| `npm run check` | Svelte と TypeScript の型チェック |
| `npm run build:dmg` | 他人に渡せる署名済み `.dmg` |
| `npm run build:sidecar` | 翻訳ワーカーのみ |
| `npm run build:icons` | `scripts/make-icon.py` からアイコンを再生成 |
| `npm run licenses` | ロックファイルから `THIRD-PARTY-NOTICES.md` を再生成する |

CI はありません。whisper.cpp と llama.cpp をソースからコンパイルし、テスト環境がシステム
出力経由で音声を再生するため、すべてのチェックはローカルコマンドです。一覧は
[CONTRIBUTING.md](../../CONTRIBUTING.md) にあります。

## テスト

ユニットテストは純粋なロジックを対象とします。[tests/](../../tests/README.md) の
スクリプトはシステム出力から音声を再生し、実際のパイプラインの出力を採点します-認識・
翻訳・レイテンシをまとめて評価します。

```bash
npm run build:sidecar
cargo test --manifest-path src-tauri/Cargo.toml
```

1 行目は一度だけ必要で、その後は `cargo clean` のあとだけです。Tauri は翻訳ワーカーを
サイドカーとして同梱するため、そのバイナリがない間はビルドスクリプトが `src-tauri` の
ビルド自体を拒否します。新しいクローンでは `cargo test` 単体が
`resource path 'binaries/marswind-translator-…' doesn't exist` で止まります。
`npm run` のビルドコマンドはこの手順を代わりに行いますが、`cargo` を直接呼ぶ場合は行いません。

パイプラインのスクリプトには、ビルドして署名したバンドルとインストール済みのモデルが必要です。

```bash
npm run dev:macos
tests/run-capture.sh
tests/run-asr.sh
tests/run-pipeline.sh
```

このコーパスでの 1 回の実行は単語誤り率で 20 ポイントほど振れるため、単独の数値には
意味がありません。複数回の中央値を比較し、スコアだけでなく文字起こしそのものを読んで
ください。

## プライバシー

- 音声は**メモリ上**で取り込み・リサンプリング・認識されます。ディスクに書き出される
  ことも、どこかへ送られることもありません。
- ネットワーク通信は、あなたが要求したモデルのダウンロードだけです。インストール後は
  一切通信しません。
- テレメトリも解析もクラッシュレポートもアカウントもありません。
- 文字起こしはアプリのデータディレクトリだけに書き込まれます（履歴表示のため）。
  アプリ内から削除できます。

## コントリビュート

バグ報告・アイデア・プルリクエストを歓迎します。
[CONTRIBUTING.md](../../CONTRIBUTING.md) にセットアップ、チェック項目、コミット規約、
レビューの観点をまとめています。大きめの変更を始める前に issue を立ててください-一見
明らかな改善のいくつかは、すでに試して測定のうえ元に戻されています。

- [行動規範](../../CODE_OF_CONDUCT.md)
- [セキュリティポリシー](../../SECURITY.md)-脆弱性は issue ではなく非公開で報告して
  ください

## 使用技術

| | | |
|---|---|---|
| [whisper.cpp](https://github.com/ggml-org/whisper.cpp) | MIT | 音声認識と、それに含まれる Silero VAD の実装 |
| [llama.cpp](https://github.com/ggml-org/llama.cpp) | MIT | 翻訳、別プロセスで動作 |
| [ggml](https://github.com/ggml-org/ggml) | MIT | 両者の下にあるテンソルライブラリと Metal バックエンド |
| [whisper-rs](https://codeberg.org/tazz4843/whisper-rs) | Unlicense | whisper.cpp の Rust バインディング |
| [llama-cpp-2](https://github.com/utilityai/llama-cpp-rs) | MIT / Apache-2.0 | llama.cpp の Rust バインディング |
| [Silero VAD](https://github.com/snakers4/silero-vad) | MIT | フレーズの区切りを見つけるモデル |
| [Tauri](https://tauri.app) | MIT / Apache-2.0 | ウィンドウとプロセス境界 |
| [Svelte](https://svelte.dev) | MIT | インターフェース |
| [rubato](https://github.com/HEnquist/rubato) | MIT | whisper の手前にある FFT リサンプラー |

依存関係の全体と各パッケージのライセンスは
[THIRD-PARTY-NOTICES.md](../../THIRD-PARTY-NOTICES.md) にあります。ロックファイルから
生成され、ライセンス本体と並べてアプリの中に同梱されます。

**モデルはこの対象外です。** モデルは利用者の求めに応じて
[Hugging Face](https://huggingface.co) から取得され、公開者の条件をそのまま引き継ぎ
ます。whisper と Silero のモデルは MIT、Qwen3 は Apache-2.0、Gemma 3 は
[Google 独自の条件](https://ai.google.dev/gemma/terms)で、これはオープンソース
ライセンスではなく、出力の用途にも条件が付きます。設定の各行は、ダウンロードが
始まる前にライセンスを示します。詳細は [docs/MODELS.md](../MODELS.md) にあります。

## ライセンス

MIT - [LICENSE](../../LICENSE) を参照してください。これはこのリポジトリについての
ものでモデルは対象外であり、上の一覧はそれが指し示すライセンスの代わりにはなりません。
