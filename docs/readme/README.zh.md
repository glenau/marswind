<div align="center">

<img src="../../src-tauri/icons/128x128@2x.png" alt="" width="128" height="128">

# Marswind

**为电脑播放的声音提供实时字幕与翻译。完全离线。**

[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](../../LICENSE)
[![Platform: macOS 14.4+](https://img.shields.io/badge/platform-macOS%2014.4%2B-lightgrey.svg)](#平台支持)
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
**中文** ·
[日本語](README.ja.md) ·
[한국어](README.ko.md)

<img src="../screenshot.png" alt="Marswind 窗口：左侧为原文，右侧为西班牙语译文" width="900">

</div>

Marswind 会监听电脑正在播放的一切声音-YouTube 视频、Google Meet、Teams 或 Zoom
通话、本地视频文件-识别其中的语音，并在说话的同时把它翻译成你选择的语言。

无需 API 密钥、无需账号、无需联网。模型只下载一次，之后在本地运行；音频始终留在
内存中，从不写入磁盘，也不会发送到任何地方。

## 它能做什么

- **捕获系统音频**，无需虚拟音频驱动-可以是电脑播放的全部声音，也可以只是浏览器
  这样的单个应用
- **识别语音**，在 GPU 上通过 whisper.cpp 运行：字幕随着话音逐步增长，而不会在读者
  眼皮底下被改写
- **边说边翻译**-词语一旦被确定就送去翻译，而不是等整句结束，译文也逐词返回
- **在应用内管理模型**：六个识别模型和三个翻译模型，全部为 MIT 或 Apache-2.0，下载时
  显示进度并做 SHA-256 校验
- **记录每一次会话**-可以回看，并导出为文本、字幕（`.srt`）或带时间信息的 JSON
- **附带示例音频**，无需另找视频即可试用
- **支持十三种界面语言**-与可翻译的语言相同-提供浅色与深色主题，字号设置会缩放
  整个界面而不仅仅是文字

### 语言

英语、俄语、德语、西班牙语、法语、意大利语、葡萄牙语、波兰语、土耳其语、乌克兰语、
中文、日语和韩语，既可作为翻译目标语言，也可作为窗口本身的语言。默认情况下由音频
自动判断所说的语言，识别能力覆盖 whisper 支持的全部语言。

## 工作原理

```
系统音频  →  重采样为 16 kHz 单声道  →  语音活动检测（Silero）
         →  语音识别（whisper.cpp）
         →  翻译（llama.cpp，独立进程）
         →  转录：左侧原文，右侧译文
```

界面之下的一切都用 Rust 编写并运行在专用线程上，翻译则位于独立的可执行文件中，因为
whisper.cpp 与 llama.cpp 无法共享同一个进程。设计与取舍记录在
[docs/ARCHITECTURE.md](../ARCHITECTURE.md)。

在 Apple Silicon 上以默认模型测得，语料为 [tests/](../../tests/README.md) 中的合成
素材-每段各跑三次取中位数：第一条字幕出现在片段开始后约 6 秒，此后每 2-3 秒刷新
一条；词错误率在朗读清晰的片段上为 4%，在满是专有名词和数字的片段上为 23%。识别不是
确定性的，单次运行的波动约有二十个百分点，所以这些是中位数而非结论；数字如何得出，
记录在测试台旁边。

## 平台支持

| 平台 | 状态 |
|---|---|
| **macOS 14.4+** | 已支持-Core Audio process taps、Metal |
| **Windows** | 开发中-WASAPI loopback |
| **Linux** | 开发中-PipeWire |

应用目前已能在 Windows 和 Linux 上构建并启动，但那里的音频捕获会报告自身不可用，也
就是一个没有东西可听的窗口。捕获之上的所有部分都与平台无关，且已经可用。

在任何平台上都**不需要** BlackHole 之类的虚拟音频驱动：捕获走的是系统原生 API。

## 系统要求

| | |
|---|---|
| macOS | 14.4 或更高，Apple Silicon 或 Intel |
| 内存 | 仅识别需 8 GB，加上翻译需 16 GB |
| 磁盘 | 所选模型占 0.1-6.5 GB |
| 构建所需 | [Rust](https://rustup.rs)、[Node.js](https://nodejs.org) 20+、cmake（`brew install cmake`） |

## 安装

### 下载

[最新发行版](https://github.com/glenau/marswind/releases/latest)附有 `.dmg`。
打开它，把 Marswind 拖进「应用程序」即可-约 13 MB，因为模型是之后才下载的，而且只下
你选的那几个。

**macOS 第一次会拒绝打开它。** 镜像已签名但未经过公证：这个项目背后没有付费的
Developer ID 证书，而 Gatekeeper 会把没有证书的一切都当作来源不明。绕过的办法：

1. 打开应用，让系统拦下它。点 **Done**，不要点「Move to Bin」。
2. 打开**系统设置 → 隐私与安全性**，向下滚动到**安全性**一节。那里会有一行说明
   Marswind 已被阻止，旁边是**仍要打开**按钮。
3. 点它，用触控 ID 或密码验证，再确认一次。

macOS 只问一次，之后就记住了。该按钮只在一次被拦截的启动之后出现，大约保留一小时；
如果不在，再打开一次应用即可。

右键点击应用再选「打开」是以前的快捷做法，在 macOS 14 上仍然有效。macOS 15 移除了
它，所以经由系统设置的路径才是到处都行得通的那条。

### 或者自己构建

```bash
git clone https://github.com/glenau/marswind.git
cd marswind
npm install
npm run install:macos
```

这会构建翻译工作进程、构建 release 包、进行 ad-hoc 签名，并复制到
`/Applications/Marswind.app`。首次构建需要几分钟-whisper.cpp 和 llama.cpp 会从源码
编译。除此之外不需要别的：没有子模块要拉取，没有库要手动安装，也没有模型
要提前下载。

### 首次运行

1. `open /Applications/Marswind.app`
2. macOS 会请求**录音**权限。请允许-否则应用什么也听不到。如果之前拒绝了，可以在
   系统设置 → 隐私与安全性 → 录音 中重新授予。
3. 打开**设置**，下载一个识别模型和一个翻译模型。16 GB 及以上的机器默认是
   `Large v3 Turbo (compressed)` 和 `Qwen3 4B Instruct`；`Small` 和 `Qwen3 1.7B`
   能装进 8 GB。约 3 GB 的下载量，每个都会对照公布的校验和验证。每一行都标明了
   权重所用的许可证-见 [docs/MODELS.md](../MODELS.md)。
4. 按**开始聆听**，然后播放带人声的内容。设置里备有四段示例音频，省得你去找视频。

关于自行构建的副本，有两点需要知道：

- **它是 ad-hoc 签名的。** 对同一次构建来说签名是稳定的，因此录音权限会保留-但重新
  构建会产生新的身份，macOS 会再次询问。能终结这一点的是 Developer ID 证书，目前还
  没有。
- **运行期间不要移动应用。** 更新时重新运行 `npm run install:macos`，它会原地替换
  `/Applications/Marswind.app`。

### 更新

**设置 → 关于 → 检查更新。** 应用会向 GitHub 询问是否有更新的版本；如果有，就把镜像
下载到「下载」，用旁边发布的校验和核对，然后在访达中显示。安装还是第一次那样拖一下。

没有任何东西会自己去检查：没有定时器，启动时也不检查，应用不会发出你没有按过按钮的
网络请求。

自己构建的副本按原样更新：再执行一次 `npm run install:macos`。

### 制作磁盘映像

```bash
npm run build:dmg
```

构建发行版包、签名，并用 `hdiutil` 打包成
`src-tauri/target/Marswind-<版本>-<架构>.dmg`-与随发行版一同附上的是同一个镜像，
也带着上面提到的同一条 Gatekeeper 注意事项。围绕它的检查清单见
[docs/RELEASING.md](../RELEASING.md)。

## 开发

`tauri dev` 生成的是没有 `Info.plist`、也没有签名的裸可执行文件，Core Audio 的
process tap 在这种形态下无法工作。请改用下面的命令-它会构建调试包、签名并启动：

```bash
npm run dev:macos
```

| 命令 | 作用 |
|---|---|
| `npm run dev:macos` | 构建、签名并启动调试包 |
| `npm run install:macos` | 构建 release 包并安装 |
| `npm run check` | Svelte 与 TypeScript 类型检查 |
| `npm run build:dmg` | 生成可交给他人的已签名 `.dmg` |
| `npm run build:sidecar` | 单独构建翻译工作进程 |
| `npm run build:icons` | 由 `scripts/make-icon.py` 重绘应用图标 |
| `npm run build:social` | 重绘 GitHub 在分享链接时展示的预览卡片 |
| `npm run licenses` | 依据 lock 文件重新生成 `THIRD-PARTY-NOTICES.md` |

没有 CI：whisper.cpp 和 llama.cpp 从源码编译，而测试台需要通过系统输出播放音频，因此
每一项检查都是本地命令。[CONTRIBUTING.md](../../CONTRIBUTING.md) 中列出了它们。

## 测试

单元测试覆盖纯逻辑；[tests/](../../tests/README.md) 中的脚本通过系统输出播放音频，
并对真实流水线的输出打分-识别、翻译与延迟一并衡量。

```bash
npm run build:sidecar
cargo test --manifest-path src-tauri/Cargo.toml
```

第一行只需执行一次，之后仅在 `cargo clean` 后才需要。Tauri 把翻译工作进程作为 sidecar
打包，因此在该二进制不存在时，它的构建脚本会拒绝构建 `src-tauri`-在全新克隆上单独运行
`cargo test` 会停在 `resource path 'binaries/marswind-translator-…' doesn't exist`。
任何 `npm run` 构建命令都会替你完成这一步，直接调用 `cargo` 则不会。

流水线脚本需要已构建并签名的包，以及已安装的模型：

```bash
npm run dev:macos
tests/run-capture.sh
tests/run-asr.sh
tests/run-pipeline.sh
```

在该语料上单次运行的词错误率会浮动约二十个百分点，因此孤立的一个数字毫无意义。请比较
多次运行的中位数，并且不只看分数，也要读转录文本。

## 隐私

- 音频在**内存中**捕获、重采样和识别。它从不写入磁盘，也不会发送到任何地方。
- 唯一的网络流量是你按下按钮才发生的：下载模型，或检查更新。没有定时任务，启动时也不联网。
- 没有遥测、没有分析、没有崩溃上报、没有账号。
- 转录内容只写入应用数据目录，以便「历史」视图有内容可显示。可以在应用内删除。

## 参与贡献

欢迎提交问题报告、想法和 pull request。
[CONTRIBUTING.md](../../CONTRIBUTING.md) 介绍了环境搭建、检查项、提交规范以及代码评审
关注的内容。开始较大改动前请先开一个 issue-有若干看似显而易见的改进已经试过并被回退，
相关测量结果都有记录。

- [行为准则](../../CODE_OF_CONDUCT.md)
- [安全策略](../../SECURITY.md)-请私下报告漏洞，不要发在 issue 中

## 构建于

| | | |
|---|---|---|
| [whisper.cpp](https://github.com/ggml-org/whisper.cpp) | MIT | 语音识别，以及随之而来的 Silero VAD 实现 |
| [llama.cpp](https://github.com/ggml-org/llama.cpp) | MIT | 翻译，运行在独立进程里 |
| [ggml](https://github.com/ggml-org/ggml) | MIT | 两者底下的张量库与 Metal 后端 |
| [whisper-rs](https://codeberg.org/tazz4843/whisper-rs) | Unlicense | whisper.cpp 的 Rust 绑定 |
| [llama-cpp-2](https://github.com/utilityai/llama-cpp-rs) | MIT / Apache-2.0 | llama.cpp 的 Rust 绑定 |
| [Silero VAD](https://github.com/snakers4/silero-vad) | MIT | 找出短语边界的模型 |
| [Tauri](https://tauri.app) | MIT / Apache-2.0 | 窗口与进程边界 |
| [Svelte](https://svelte.dev) | MIT | 界面 |
| [rubato](https://github.com/HEnquist/rubato) | MIT | whisper 之前的 FFT 重采样器 |

完整的依赖树，连同每个包的许可证，都在
[THIRD-PARTY-NOTICES.md](../../THIRD-PARTY-NOTICES.md) 里-由 lock 文件生成，并
与许可证本身一起打包进应用。

**这些都不涵盖模型。** 模型是按你的要求从 [Hugging Face](https://huggingface.co)
下载的，各自保留发布者的条款，而进入目录的只有那些条款可以不读就接受的：whisper 与
Silero 模型是 MIT，Qwen3 是 Apache-2.0。设置里的每一行都会在下载开始前写明其许可证。
详情见 [docs/MODELS.md](../MODELS.md)。

## 许可证

MIT-见 [LICENSE](../../LICENSE)。这只涵盖本仓库，不涵盖模型；上面的声明也不能
替代它所指向的那些许可证。
