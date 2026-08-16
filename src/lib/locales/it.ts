import type { Dictionary } from "./en";

const it: Dictionary = {
  strings: {
    "stage.audio": "Audio",
    "stage.recognition": "Riconoscimento",
    "stage.translation": "Traduzione",
    "stage.notCapturing": "Nessuna acquisizione",
    "stage.notRunning": "Non attivo",
    "meter.level": "Livello in ingresso",
    "action.start": "Inizia ad ascoltare",
    "action.stop": "Ferma",
    "action.working": "Avvio…",
    "action.history": "Cronologia",
    "action.settings": "Impostazioni",
    "action.clear": "Svuota",
    "action.dismiss": "Nascondi",
    "action.refresh": "Aggiorna",
    "action.delete": "Elimina",
    "action.play": "Riproduci",
    "action.text": "Testo",
    "action.hideText": "Nascondi il testo",
    "action.install": "Installa",
    "action.remove": "Rimuovi",
    "action.cancel": "Annulla",
    "action.github": "Il progetto su GitHub",

    "transcript.original": "Originale",
    "transcript.translation": "Traduzione",
    "transcript.into": "Tradotto in",
    "transcript.toLanguage": "in",
    "transcript.emptySource":
      "Premi «Inizia ad ascoltare» e riproduci qualcosa di parlato - oppure uno degli esempi nelle impostazioni. Qui comparirà ciò che viene detto.",
    "transcript.emptyTarget":
      "Qui compare la traduzione, un pezzo di frase alla volta, mentre chi parla sta ancora parlando.",
    "transcript.translating": "traduzione…",
    "transcript.notTranslated": "- non tradotto",
    "transcript.toBottom": "Segui l'ultima riga",

    "settings.general": "Generale",
    "settings.models": "Modelli",
    "settings.about": "Informazioni",
    "settings.audioTitle": "Audio",
    "settings.audioNote":
      "Da dove arriva il suono. Tutto ciò che questo Mac riproduce, oppure una singola app.",
    "settings.audio": "Sorgente audio",
    "settings.recognition": "Riconoscimento vocale",
    "settings.recognitionNote":
      "Trasforma in testo ciò che il computer sta riproducendo. Serve un modello di riconoscimento.",
    "settings.translation": "Traduzione",
    "settings.translationNote":
      "Porta il testo riconosciuto in un'altra lingua. Serve un modello di traduzione, diverso da quello sopra.",
    "settings.on": "Attiva",
    "settings.showOriginal": "Mostra l'originale accanto alla traduzione",
    "settings.locked": "Ferma l'ascolto per modificare queste impostazioni.",
    "settings.noModel": "Nessun modello installato",
    "settings.needVad":
      "Installa qui sotto il modello di rilevamento della voce: il riconoscimento ne ha bisogno.",
    "settings.needAsr": "Installa qui sotto un modello di riconoscimento.",
    "settings.needMt": "Installa qui sotto un modello di traduzione.",
    "settings.detect": "Rileva automaticamente",
    "settings.model": "Modello",
    "settings.interface": "Interfaccia",
    "settings.interfaceNote":
      "L'aspetto della finestra. Niente di tutto questo cambia ciò che l'app fa con l'audio.",
    "settings.language": "Lingua",
    "settings.theme": "Tema",
    "settings.textSize": "Dimensione del testo",
    "settings.samples": "Provalo senza cercare un video",
    "settings.samplesNote":
      "Riproduce uno spezzone registrato dagli altoparlanti, così fa la stessa strada di qualsiasi altro suono. Prima avvia l'ascolto.",

    "theme.dark": "Scuro",
    "theme.light": "Chiaro",
    "theme.system": "Come il sistema",

    "about.title": "Informazioni",
    "about.tagline":
      "Sottotitoli e traduzione dal vivo per tutto ciò che questo computer riproduce. Riconoscimento e traduzione girano qui: nessun account, nessuna chiave API e niente che esca dalla macchina.",
    "about.version": "Versione",
    "about.runtime": "Runtime",
    "about.license": "Licenza",
    "about.source": "Codice sorgente",
    "about.issues": "Segnala un problema",
    "about.licenseFile": "Leggi la licenza",
    "about.notices": "Licenze di terze parti",
    "about.built": "Realizzato con",
    "about.how": "Come funziona",
    "about.howCapture":
      "Il suono viene preso da macOS stessa, con un tap di processo di Core Audio: proprio ciò che il Mac sta già riproducendo, tutto insieme o una sola app. Nessun driver audio virtuale, nessun microfono, e non cambia nulla di ciò che senti.",
    "about.howPipeline":
      "Quel flusso viene mixato in mono a 16 kHz e resta in memoria. Silero VAD lo taglia in frasi, whisper.cpp le trasforma in testo sulla GPU e llama.cpp traduce in un processo separato mentre chi parla sta ancora parlando. L'audio in sé non viene mai scritto su disco e non esce mai da questo Mac.",

    "models.title": "Modelli",
    "models.note":
      "Tutto gira su questa macchina, quindi ogni modello è un file su questo disco. Installane uno per il riconoscimento e uno per la traduzione.",
    "models.onDisk": "sul disco",
    "models.forRecognition": "riconoscimento",
    "models.forTranslation": "traduzione",
    "models.forVad": "rilevamento voce",
    "models.recommended": "consigliato",
    "models.required": "necessario",
    "models.installed": "installato",
    "models.of": "di",

    "history.empty":
      "Ancora nessuna sessione. Ogni volta che avvii l'ascolto, ciò che viene riconosciuto e tradotto finisce qui.",
    "history.pick": "Scegli una sessione a sinistra.",
    "history.loading": "Caricamento…",
    "history.rows": "righe",
    "history.words": "parole",
    "history.exportText": "Esporta il testo",
    "history.exportSrt": "Esporta i sottotitoli",
    "history.exportJson": "Esporta JSON",
    "history.saved": "Salvato in",
    "history.noModel": "nessun modello di riconoscimento",
    "history.notTranslated": "non tradotto",
    "history.segment": "segmento",
    "history.segments": "segmenti",

    "size.small": "Piccolo",
    "size.medium": "Medio",
    "size.large": "Grande",
    "size.huge": "Molto grande",
  },
  phrases: {
    idle: [
      "Ciao. Traduciamo qualcosa?",
      "Metti qualsiasi cosa in cui si parli",
      "Pronto quando vuoi",
      "Che silenzio. Io ascolto lo stesso",
      "Un clic e le parole cominciano ad arrivare",
      "Niente di ciò che riproduci esce da questo Mac",
      "Cosa guardiamo oggi?",
      "Un podcast, una lezione, una chiamata: per me è uguale",
      "Aspetto la prima frase",
      "Avvia l'ascolto e si parte",
    ],
    live: [
      "Ascolto con attenzione…",
      "Provo a distinguere le parole…",
      "Prendo le frasi al volo…",
      "Quella era una parola, di sicuro…",
      "Sfoglio il mio dizionario…",
      "Sto al passo, più o meno…",
      "Traduco più in fretta di quanto penso…",
      "Tutto orecchie, niente distrazioni…",
      "Ci sono quasi con questa frase…",
      "Seguo il filo…",
    ],
  },
};

export default it;
