<div align="center">

<img src="../../src-tauri/icons/128x128@2x.png" alt="" width="128" height="128">

# Marswind

**Legendas e tradução ao vivo do áudio do seu computador. Totalmente offline.**

[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](../../LICENSE)
[![Platform: macOS 14.4+](https://img.shields.io/badge/platform-macOS%2014.4%2B-lightgrey.svg)](#plataformas)
[![Version](https://img.shields.io/badge/version-0.1.0-brightgreen.svg)](#)

[English](../../README.md) ·
[Русский](README.ru.md) ·
[Deutsch](README.de.md) ·
[Español](README.es.md) ·
[Français](README.fr.md) ·
[Italiano](README.it.md) ·
**Português** ·
[Polski](README.pl.md) ·
[Türkçe](README.tr.md) ·
[Українська](README.uk.md) ·
[中文](README.zh.md) ·
[日本語](README.ja.md) ·
[한국어](README.ko.md)

<img src="../screenshot.png" alt="A janela do Marswind: o original à esquerda, a tradução para espanhol à direita" width="900">

</div>

O Marswind escuta o que está a tocar na sua máquina - um vídeo do YouTube, uma
chamada no Google Meet, Teams ou Zoom, um ficheiro de vídeo local - reconhece a
fala e traduz para o idioma que escolher enquanto a pessoa fala.

Sem chaves de API, sem contas, sem internet. Os modelos são descarregados uma
vez e depois correm localmente; o áudio fica em memória, nunca é escrito em
disco nem enviado para lado nenhum.

## O que faz

- **Captura o áudio do sistema** sem driver de áudio virtual - tudo o que a
  máquina reproduz, ou uma única aplicação como o navegador
- **Reconhece a fala** com whisper.cpp na GPU: as legendas crescem à medida que se
  fala, em vez de serem reescritas debaixo de quem lê
- **Traduz enquanto se fala** - as palavras vão para o tradutor assim que ficam
  fixas, não no fim da frase, e a tradução chega palavra a palavra
- **Gere os modelos** a partir da própria aplicação: sete de reconhecimento e
  cinco de tradução, descarregados com progresso e verificação SHA-256
- **Grava cada sessão** - podem ser consultadas depois e exportadas como texto,
  legendas (`.srt`) ou JSON com os tempos correspondentes
- **Traz clipes de exemplo**, para experimentar sem ter de procurar um vídeo
- **Fala treze idiomas** - os mesmos para os quais traduz - em tema claro ou
  escuro, com um tamanho de texto que escala toda a interface e não apenas a letra

### Idiomas

Inglês, russo, alemão, espanhol, francês, italiano, português, polaco, turco,
ucraniano, chinês, japonês e coreano, tanto como idiomas de destino como idioma
da própria janela. O idioma falado é deduzido do áudio por omissão, e o
reconhecimento cobre tudo o que o whisper cobre.

## Como funciona

```
Áudio do sistema  →  reamostragem para 16 kHz mono  →  deteção de voz (Silero)
                  →  reconhecimento de fala (whisper.cpp)
                  →  tradução (llama.cpp, num processo separado)
                  →  transcrição: original à esquerda, tradução ao lado
```

Tudo abaixo da interface está em Rust e corre em threads dedicadas, e a tradução
vive num binário separado porque o whisper.cpp e o llama.cpp não podem partilhar
um processo. O desenho e as suas razões estão em
[docs/ARCHITECTURE.md](../ARCHITECTURE.md).

Medido em Apple Silicon com os modelos padrão, sobre o corpus sintético em
[tests/](../../tests/README.md) - medianas de três execuções por clipe: a
primeira legenda cerca de 6 segundos após o início, uma nova a cada 2-3
segundos, e uma taxa de erro por palavra entre 4 % numa leitura limpa e 23 % num
clipe cheio de nomes próprios e números. O reconhecimento não é determinista e
uma única execução varia uns vinte pontos, portanto são medianas e não
resultados; como os números são produzidos está documentado junto à bancada de
testes.

## Plataformas

| Plataforma | Estado |
|---|---|
| **macOS 14.4+** | Suportado - Core Audio process taps, Metal |
| **Windows** | Em desenvolvimento - WASAPI loopback |
| **Linux** | Em desenvolvimento - PipeWire |

A aplicação já compila e arranca em Windows e Linux, mas aí a captura de áudio
declara-se indisponível: uma janela sem nada para ouvir. Tudo o que está acima
da captura já é independente da plataforma.

Um driver de áudio virtual como o BlackHole **não** é necessário em nenhuma
plataforma: a captura usa as APIs nativas do sistema.

## Requisitos

| | |
|---|---|
| macOS | 14.4 ou mais recente, Apple Silicon ou Intel |
| Memória | 8 GB só para o reconhecimento, 16 GB com tradução |
| Disco | 0,5-4,5 GB para os modelos escolhidos |
| Para compilar | [Rust](https://rustup.rs), [Node.js](https://nodejs.org) 20+, cmake (`brew install cmake`) |

## Instalação

### Baixar

A [versão mais recente](https://github.com/glenau/marswind/releases/latest) traz
um `.dmg`. Abra-o, arraste o Marswind para Aplicações e pronto - cerca de 13 MB,
já que os modelos vêm depois e só os que você escolher.

**O macOS vai recusar abri-lo na primeira tentativa.** A imagem é assinada mas não
notarizada: não há certificado Developer ID pago por trás deste projeto, e o
Gatekeeper trata como desconhecido tudo o que não o tenha. O caminho:

1. Abra o app e deixe que seja bloqueado. Pressione **Done**, não "Move to Bin".
2. **Ajustes do Sistema → Privacidade e segurança**, role até **Segurança**. Há
   uma linha dizendo que o Marswind foi bloqueado, e ao lado um botão
   **Abrir mesmo assim**.
3. Pressione, autentique com Touch ID ou senha e confirme mais uma vez.

O macOS pergunta uma vez e guarda a resposta. O botão só aparece depois de uma
abertura bloqueada e dura cerca de uma hora; se não estiver lá, abra o app de
novo.

Clicar com o botão direito no app e escolher Abrir era o atalho anterior para
isso e ainda funciona no macOS 14. O macOS 15 removeu, então o caminho pelos
ajustes é o que funciona em todo lugar.

### Ou compilar

```bash
git clone https://github.com/glenau/marswind.git
cd marswind
npm install
npm run install:macos
```

Isto compila o worker de tradução, compila o bundle de release, assina-o em modo
ad-hoc e copia-o para `/Applications/Marswind.app`. A primeira compilação demora
vários minutos - o whisper.cpp e o llama.cpp são compilados a partir do código.
Nada mais é preciso: sem submódulos para buscar, sem bibliotecas para instalar à
mão e sem modelos para baixar antes.

### Primeiro arranque

1. `open /Applications/Marswind.app`
2. O macOS pede a permissão de **Gravação de áudio**. Aceite - sem ela a aplicação
   não ouve nada. Se foi recusada, pode ser concedida de novo em Definições do
   Sistema → Privacidade e segurança → Gravação de áudio.
3. Abra os **Ajustes** e baixe um modelo de reconhecimento e um de tradução.
   `Large v3 Turbo (compressed)` e `Qwen3 4B Instruct` são os padrões a partir de
   16 GB; `Small` e `Qwen3 1.7B` cabem em 8 GB. Cerca de 3 GB de download,
   conferidos contra uma soma de verificação publicada. Cada linha indica a
   licença dos seus pesos - veja [docs/MODELS.md](../MODELS.md).
4. Toque em **Começar a ouvir** e reproduza algo com fala. Há quatro clipes de
   exemplo nos ajustes, se preferir não ir atrás de um vídeo.

Duas coisas a saber sobre uma cópia compilada por si:

- **Está assinada em modo ad-hoc.** A assinatura é estável para uma dada
  compilação, por isso a permissão de áudio persiste - mas recompilar produz uma
  identidade nova e o macOS pergunta outra vez. É um certificado Developer ID que
  acaba com isso, e ainda não existe nenhum.
- **Não mova a aplicação enquanto está a correr.** Para a atualizar, volte a
  executar `npm run install:macos`; substitui `/Applications/Marswind.app` no
  lugar.

### Criar uma imagem de disco

```bash
npm run build:dmg
```

Compila o pacote de release, assina-o e empacota em
`src-tauri/target/Marswind-<versão>-<arq>.dmg` - a mesma imagem anexada a uma
publicação, com a mesma ressalva sobre o Gatekeeper acima. A lista de
verificação está em [docs/RELEASING.md](../RELEASING.md).

## Desenvolvimento

O `tauri dev` produz um executável nu sem `Info.plist` e sem assinatura, e os
process taps do Core Audio recusam-se a funcionar nessa forma. Use antes isto -
compila um bundle de debug, assina-o e lança-o:

```bash
npm run dev:macos
```

| Comando | O que faz |
|---|---|
| `npm run dev:macos` | compilar, assinar e lançar um bundle de debug |
| `npm run install:macos` | compilar um bundle de release e instalá-lo |
| `npm run check` | tipos de Svelte e TypeScript |
| `npm run build:dmg` | um `.dmg` assinado para dar a alguém |
| `npm run build:sidecar` | só o worker de tradução |
| `npm run build:icons` | redesenhar o ícone a partir de `scripts/make-icon.py` |
| `npm run licenses` | regerar `THIRD-PARTY-NOTICES.md` a partir dos lockfiles |

Não há CI: o whisper.cpp e o llama.cpp são compilados a partir do código e a
bancada de testes reproduz áudio pela saída do sistema, por isso cada
verificação é um comando local. O [CONTRIBUTING.md](../../CONTRIBUTING.md)
lista-as.

## Testes

Os testes unitários cobrem a lógica pura; os scripts em
[tests/](../../tests/README.md) reproduzem áudio pela saída do sistema e pontuam
o que sai do pipeline real - reconhecimento, tradução e latência em conjunto.

```bash
npm run build:sidecar
cargo test --manifest-path src-tauri/Cargo.toml
```

A primeira linha é necessária uma vez, e depois só após um `cargo clean`. O
Tauri empacota o worker de tradução como sidecar, então o script de build se
recusa a compilar `src-tauri` enquanto o binário não existir: num clone recém-
feito, `cargo test` sozinho para em
`resource path 'binaries/marswind-translator-…' doesn't exist`. Qualquer comando
`npm run` faz esse passo por você; chamar `cargo` direto, não.

Os scripts do pipeline precisam de um pacote compilado e assinado, e de modelos
instalados:

```bash
npm run dev:macos
tests/run-capture.sh
tests/run-asr.sh
tests/run-pipeline.sh
```

Uma única execução sobre o corpus varia cerca de vinte pontos de taxa de erro,
por isso um número isolado não significa nada. Compare medianas entre execuções
e leia as transcrições, não apenas as pontuações.

## Privacidade

- O áudio é capturado, reamostrado e reconhecido **em memória**. Nunca é escrito
  em disco nem enviado para lado nenhum.
- O único tráfego de rede é a transferência dos modelos que pedir. Depois de
  instalados, a aplicação não gera nenhum.
- Sem telemetria, sem analítica, sem relatórios de falhas, sem conta.
- As transcrições são escritas apenas na pasta de dados da aplicação, para que a
  vista de Histórico tenha algo para mostrar. Apagam-se dentro da aplicação.

## Contribuir

Relatórios de erros, ideias e pull requests são bem-vindos. O
[CONTRIBUTING.md](../../CONTRIBUTING.md) cobre a configuração, as verificações,
a convenção de commits e o que a revisão procura. Abra uma issue antes de
começar algo grande - várias melhorias óbvias já foram tentadas e revertidas,
com as medições registadas.

- [Código de conduta](../../CODE_OF_CONDUCT.md)
- [Política de segurança](../../SECURITY.md) - comunique vulnerabilidades em
  privado, não numa issue

## Construído sobre

| | | |
|---|---|---|
| [whisper.cpp](https://github.com/ggml-org/whisper.cpp) | MIT | reconhecimento, e com ele a implementação do Silero VAD |
| [llama.cpp](https://github.com/ggml-org/llama.cpp) | MIT | tradução, em processo próprio |
| [ggml](https://github.com/ggml-org/ggml) | MIT | a biblioteca de tensores e o backend Metal sob ambos |
| [whisper-rs](https://codeberg.org/tazz4843/whisper-rs) | Unlicense | a ligação Rust para whisper.cpp |
| [llama-cpp-2](https://github.com/utilityai/llama-cpp-rs) | MIT / Apache-2.0 | a ligação Rust para llama.cpp |
| [Silero VAD](https://github.com/snakers4/silero-vad) | MIT | o modelo que encontra as fronteiras de frase |
| [Tauri](https://tauri.app) | MIT / Apache-2.0 | a janela e a fronteira entre processos |
| [Svelte](https://svelte.dev) | MIT | a interface |
| [rubato](https://github.com/HEnquist/rubato) | MIT | o reamostrador FFT à frente do whisper |

Toda a árvore de dependências, com a licença de cada pacote, está em
[THIRD-PARTY-NOTICES.md](../../THIRD-PARTY-NOTICES.md) - gerada a partir dos
lockfiles e distribuída dentro do app ao lado da própria licença.

**Nada disso cobre os modelos.** Eles são baixados do
[Hugging Face](https://huggingface.co) a seu pedido e mantêm os termos de quem
os publica: os modelos whisper e Silero são MIT, o Qwen3 é Apache-2.0, e o Gemma
3 está sob os [termos do próprio Google](https://ai.google.dev/gemma/terms), que
não são uma licença de código aberto e impõem condições sobre o uso da saída.
Cada linha nos ajustes indica sua licença antes de o download começar. Os
detalhes estão em [docs/MODELS.md](../MODELS.md).

## Licença

MIT - veja [LICENSE](../../LICENSE). Isso cobre este repositório; não cobre os
modelos, e as notas acima não substituem as licenças para as quais apontam.
