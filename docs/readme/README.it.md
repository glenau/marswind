<div align="center">

<img src="../../src-tauri/icons/128x128@2x.png" alt="" width="128" height="128">

# Marswind

**Sottotitoli e traduzione dal vivo dell'audio del tuo computer. Completamente
offline.**

[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](../../LICENSE)
[![Platform: macOS 14.4+](https://img.shields.io/badge/platform-macOS%2014.4%2B-lightgrey.svg)](#piattaforme)
[![Version](https://img.shields.io/badge/version-0.1.1-brightgreen.svg)](#)

[English](../../README.md) ·
[Русский](README.ru.md) ·
[Deutsch](README.de.md) ·
[Español](README.es.md) ·
[Français](README.fr.md) ·
**Italiano** ·
[Português](README.pt.md) ·
[Polski](README.pl.md) ·
[Türkçe](README.tr.md) ·
[Українська](README.uk.md) ·
[中文](README.zh.md) ·
[日本語](README.ja.md) ·
[한국어](README.ko.md)

<img src="../screenshot.png" alt="La finestra di Marswind: l'originale a sinistra, la traduzione in spagnolo a destra" width="900">

</div>

Marswind ascolta ciò che sta suonando sulla tua macchina - un video di YouTube,
una chiamata su Google Meet, Teams o Zoom, un file video locale - riconosce il
parlato e lo traduce nella lingua che scegli mentre si parla.

Nessuna chiave API, nessun account, nessuna connessione. I modelli si scaricano
una volta e poi girano in locale; l'audio resta in memoria, non viene mai
scritto su disco né inviato da nessuna parte.

## Cosa fa

- **Cattura l'audio di sistema** senza driver audio virtuale - tutto ciò che la
  macchina riproduce, o una singola applicazione come il browser
- **Riconosce il parlato** con whisper.cpp sulla GPU: i sottotitoli crescono
  mentre si parla, invece di essere riscritti sotto chi legge
- **Traduce mentre si parla** - le parole vanno al traduttore appena sono fissate,
  non a fine frase, e la traduzione arriva parola per parola
- **Gestisce i modelli** dall'app stessa: sei di riconoscimento e tre di
  traduzione, tutti MIT o Apache-2.0, scaricati con avanzamento e verifica
  SHA-256
- **Registra ogni sessione** - consultabili in seguito ed esportabili come testo,
  sottotitoli (`.srt`) o JSON con i tempi corrispondenti
- **Include clip di esempio**, per provarlo senza andare a cercare un video
- **Parla tredici lingue** - le stesse verso cui traduce - in tema chiaro o scuro,
  con una dimensione del testo che scala tutta l'interfaccia e non solo i caratteri

### Lingue

Inglese, russo, tedesco, spagnolo, francese, italiano, portoghese, polacco,
turco, ucraino, cinese, giapponese e coreano, sia come lingue di destinazione
sia come lingua della finestra stessa. La lingua parlata viene dedotta
dall'audio per impostazione predefinita, e il riconoscimento copre tutto ciò che
copre whisper.

## Come funziona

```
Audio di sistema  →  ricampionamento a 16 kHz mono  →  rilevamento voce (Silero)
                  →  riconoscimento vocale (whisper.cpp)
                  →  traduzione (llama.cpp, in un processo separato)
                  →  trascrizione: originale a sinistra, traduzione accanto
```

Tutto ciò che sta sotto l'interfaccia è in Rust e gira su thread dedicati, e la
traduzione vive in un binario separato perché whisper.cpp e llama.cpp non
possono condividere un processo. Il progetto e le sue ragioni sono in
[docs/ARCHITECTURE.md](../ARCHITECTURE.md).

Misurato su Apple Silicon con i modelli predefiniti, sul corpus sintetico in
[tests/](../../tests/README.md) - mediane di tre esecuzioni per clip: il primo
sottotitolo circa 6 secondi dopo l'inizio, uno nuovo ogni 2-3 secondi, e un
tasso di errore per parola tra il 4 % su una lettura pulita e il 23 % su una
clip piena di nomi propri e cifre. Il riconoscimento non è deterministico e una
singola esecuzione oscilla di una ventina di punti: sono mediane, non risultati;
come nascono questi numeri è documentato accanto al banco di prova.

## Piattaforme

| Piattaforma | Stato |
|---|---|
| **macOS 14.4+** | Supportata - Core Audio process taps, Metal |
| **Windows** | In sviluppo - WASAPI loopback |
| **Linux** | In sviluppo - PipeWire |

L'app oggi compila e si avvia su Windows e Linux, ma lì la cattura audio si
dichiara non disponibile: una finestra senza nulla da ascoltare. Tutto ciò che
sta sopra la cattura è già indipendente dalla piattaforma.

Un driver audio virtuale come BlackHole **non** serve su nessuna piattaforma: la
cattura passa dalle API native del sistema.

## Requisiti

| | |
|---|---|
| macOS | 14.4 o successivo, Apple Silicon o Intel |
| Memoria | 8 GB per il solo riconoscimento, 16 GB con la traduzione |
| Disco | 0,1-6,5 GB per i modelli scelti |
| Per compilare | [Rust](https://rustup.rs), [Node.js](https://nodejs.org) 20+, cmake (`brew install cmake`) |

## Installazione

### Scaricarlo

L'[ultima release](https://github.com/glenau/marswind/releases/latest) contiene
un `.dmg`. Aprilo, trascina Marswind in Applicazioni, fatto - circa 13 MB,
perché i modelli arrivano dopo e solo quelli che scegli.

**macOS si rifiuterà di aprirlo al primo tentativo.** L'immagine è firmata ma non
notarizzata: dietro questo progetto non c'è alcun certificato Developer ID a
pagamento, e Gatekeeper considera sconosciuto tutto ciò che ne è privo. La via
d'uscita:

1. Apri l'app e lascia che venga bloccata. Premi **Done**, non "Move to Bin".
2. **Impostazioni di Sistema → Privacy e sicurezza**, scorri fino a
   **Sicurezza**. C'è una riga che dice che Marswind è stato bloccato, e accanto
   un pulsante **Apri comunque**.
3. Premilo, autenticati con Touch ID o password e conferma ancora una volta.

macOS chiede una volta sola. Il pulsante compare solo dopo un avvio bloccato e
dura circa un'ora; se non c'è, riapri l'app.

Fare clic destro sull'app e scegliere Apri era la scorciatoia di prima e
funziona ancora su macOS 14. macOS 15 l'ha rimossa, quindi il percorso dalle
impostazioni è quello che funziona ovunque.

### Oppure compilarlo

```bash
git clone https://github.com/glenau/marswind.git
cd marswind
npm install
npm run install:macos
```

Questo compila il worker di traduzione, compila il bundle di release, lo firma
in modalità ad-hoc e lo copia in `/Applications/Marswind.app`. La prima
compilazione richiede diversi minuti - whisper.cpp e llama.cpp vengono compilati
dai sorgenti. Non serve altro: nessun sottomodulo da recuperare, nessuna
libreria da installare a mano e nessun modello da scaricare in anticipo.

### Primo avvio

1. `open /Applications/Marswind.app`
2. macOS chiede il permesso di **Registrazione audio**. Accettalo - senza, l'app
   non sente nulla. Se è stato rifiutato, si concede di nuovo in Impostazioni di
   Sistema → Privacy e sicurezza → Registrazione audio.
3. Apri le **Impostazioni** e scarica un modello di riconoscimento e uno di
   traduzione. `Large v3 Turbo (compressed)` e `Qwen3 4B Instruct` sono i valori
   predefiniti da 16 GB in su; `Small` e `Qwen3 1.7B` stanno in 8 GB. Circa 3 GB
   di download, verificati contro un checksum pubblicato. Ogni riga indica la
   licenza dei suoi pesi - vedi [docs/MODELS.md](../MODELS.md).
4. Premi **Inizia ad ascoltare** e riproduci qualcosa con del parlato. Nelle
   impostazioni ci sono quattro clip di esempio, se preferisci non cercare un
   video.

Due cose da sapere su una copia compilata da te:

- **È firmata in modalità ad-hoc.** La firma è stabile per una data compilazione,
  quindi il permesso audio persiste - ma ricompilare produce una nuova identità e
  macOS lo richiede di nuovo. È un certificato Developer ID a farlo smettere, e
  non ce n'è ancora uno.
- **Non spostare l'app mentre è in esecuzione.** Per aggiornarla, riesegui
  `npm run install:macos`; sostituisce `/Applications/Marswind.app` sul posto.

### Aggiornare

**Impostazioni → Informazioni → Cerca aggiornamenti.** L'app chiede a GitHub se
esiste una versione più recente; se c'è, scarica l'immagine in Download, la
confronta con il checksum pubblicato accanto e la mostra nel Finder. Installarla
è lo stesso trascinamento della prima volta.

Niente si controlla da solo: nessun timer e nessun controllo all'avvio, perché
l'app non fa richieste di rete che non hai premuto.

Una copia compilata da te si aggiorna come è stata installata: di nuovo
`npm run install:macos`.

### Creare un'immagine disco

```bash
npm run build:dmg
```

Compila il bundle di release, lo firma e lo impacchetta in
`src-tauri/target/Marswind-<versione>-<arch>.dmg` - la stessa immagine allegata
a una release, con lo stesso avvertimento su Gatekeeper di cui sopra. La lista
di controllo è in [docs/RELEASING.md](../RELEASING.md).

## Sviluppo

`tauri dev` produce un eseguibile nudo senza `Info.plist` e senza firma, e i
process tap di Core Audio in quella forma non funzionano. Usa invece questo -
compila un bundle di debug, lo firma e lo avvia:

```bash
npm run dev:macos
```

| Comando | Cosa fa |
|---|---|
| `npm run dev:macos` | compila, firma e avvia un bundle di debug |
| `npm run install:macos` | compila un bundle di release e lo installa |
| `npm run check` | tipi Svelte e TypeScript |
| `npm run build:dmg` | un `.dmg` firmato da passare a qualcuno |
| `npm run build:sidecar` | solo il worker di traduzione |
| `npm run build:icons` | ridisegna l'icona da `scripts/make-icon.py` |
| `npm run build:social` | ridisegnare la scheda che GitHub mostra su un link |
| `npm run licenses` | rigenerare `THIRD-PARTY-NOTICES.md` dai lockfile |

Non c'è CI: whisper.cpp e llama.cpp vengono compilati dai sorgenti e il banco di
prova riproduce audio attraverso l'uscita di sistema, quindi ogni controllo è un
comando locale. [CONTRIBUTING.md](../../CONTRIBUTING.md) li elenca.

## Test

I test unitari coprono la logica pura; gli script in
[tests/](../../tests/README.md) riproducono audio attraverso l'uscita di sistema
e valutano ciò che esce dalla pipeline reale - riconoscimento, traduzione e
latenza insieme.

```bash
npm run build:sidecar
cargo test --manifest-path src-tauri/Cargo.toml
```

La prima riga serve una volta sola, e poi solo dopo un `cargo clean`. Tauri
impacchetta il worker di traduzione come sidecar, quindi il suo script di build
si rifiuta di compilare `src-tauri` finché il binario non esiste: su un clone
appena fatto `cargo test` da solo si ferma su
`resource path 'binaries/marswind-translator-…' doesn't exist`. Ogni comando
`npm run` esegue questo passaggio al posto tuo; chiamare `cargo` direttamente
no.

Gli script della pipeline richiedono un bundle compilato e firmato, e i modelli
installati:

```bash
npm run dev:macos
tests/run-capture.sh
tests/run-asr.sh
tests/run-pipeline.sh
```

Una singola esecuzione sul corpus varia di una ventina di punti di tasso di
errore, quindi un numero da solo non significa nulla. Confronta le mediane su
più esecuzioni e leggi le trascrizioni, non solo i punteggi.

## Privacy

- L'audio viene catturato, ricampionato e riconosciuto **in memoria**. Non viene
  mai scritto su disco né inviato da nessuna parte.
- L'unico traffico di rete è quello per cui premi un pulsante: scaricare un
  modello o cercare aggiornamenti. Niente parte a timer o all'avvio.
- Nessuna telemetria, nessuna analitica, nessun report di crash, nessun account.
- Le trascrizioni vengono scritte solo nella cartella dati dell'app, perché la
  vista Cronologia abbia qualcosa da mostrare. Si cancellano dall'app stessa.

## Contribuire

Segnalazioni, idee e pull request sono benvenute.
[CONTRIBUTING.md](../../CONTRIBUTING.md) copre la configurazione, i controlli,
la convenzione sui commit e cosa guarda la revisione. Apri una issue prima di
iniziare qualcosa di grosso - diversi miglioramenti ovvi sono già stati provati
e annullati, con le misurazioni annotate.

- [Codice di condotta](../../CODE_OF_CONDUCT.md)
- [Politica di sicurezza](../../SECURITY.md) - segnala le vulnerabilità in privato,
  non in una issue

## Costruito su

| | | |
|---|---|---|
| [whisper.cpp](https://github.com/ggml-org/whisper.cpp) | MIT | riconoscimento, e con esso l'implementazione di Silero VAD |
| [llama.cpp](https://github.com/ggml-org/llama.cpp) | MIT | traduzione, in un processo a parte |
| [ggml](https://github.com/ggml-org/ggml) | MIT | la libreria di tensori e il backend Metal sotto entrambi |
| [whisper-rs](https://codeberg.org/tazz4843/whisper-rs) | Unlicense | il binding Rust a whisper.cpp |
| [llama-cpp-2](https://github.com/utilityai/llama-cpp-rs) | MIT / Apache-2.0 | il binding Rust a llama.cpp |
| [Silero VAD](https://github.com/snakers4/silero-vad) | MIT | il modello che trova i confini di frase |
| [Tauri](https://tauri.app) | MIT / Apache-2.0 | la finestra e il confine tra processi |
| [Svelte](https://svelte.dev) | MIT | l'interfaccia |
| [rubato](https://github.com/HEnquist/rubato) | MIT | il ricampionatore FFT davanti a whisper |

L'intero albero delle dipendenze, con la licenza di ogni pacchetto, è in
[THIRD-PARTY-NOTICES.md](../../THIRD-PARTY-NOTICES.md) - generato dai lockfile e
incluso dentro l'app accanto alla licenza stessa.

**Niente di tutto questo copre i modelli.** Vengono scaricati da
[Hugging Face](https://huggingface.co) su tua richiesta e mantengono i termini di
chi li pubblica - e nel catalogo entrano solo quelli i cui termini si accettano
senza leggerli: i modelli whisper e Silero sono MIT, Qwen3 è Apache-2.0. Ogni
riga nelle impostazioni indica la propria licenza prima che il download cominci.
I dettagli sono in [docs/MODELS.md](../MODELS.md).

## Licenza

MIT - vedi [LICENSE](../../LICENSE). Vale per questo repository; non vale per i
modelli, e le note qui sopra non sostituiscono le licenze a cui rimandano.
