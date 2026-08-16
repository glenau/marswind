import type { Dictionary } from "./en";

const pl: Dictionary = {
  strings: {
    "stage.audio": "Dźwięk",
    "stage.recognition": "Rozpoznawanie",
    "stage.translation": "Tłumaczenie",
    "stage.notCapturing": "Brak przechwytywania",
    "stage.notRunning": "Nie działa",
    "meter.level": "Poziom sygnału",
    "action.start": "Zacznij słuchać",
    "action.stop": "Zatrzymaj",
    "action.working": "Uruchamianie…",
    "action.history": "Historia",
    "action.settings": "Ustawienia",
    "action.clear": "Wyczyść",
    "action.dismiss": "Ukryj",
    "action.refresh": "Odśwież",
    "action.delete": "Usuń",
    "action.play": "Odtwórz",
    "action.text": "Tekst",
    "action.hideText": "Ukryj tekst",
    "action.install": "Zainstaluj",
    "action.remove": "Usuń",
    "action.cancel": "Anuluj",
    "action.github": "Projekt na GitHubie",

    "transcript.original": "Oryginał",
    "transcript.translation": "Tłumaczenie",
    "transcript.into": "Tłumaczenie na",
    "transcript.toLanguage": "na",
    "transcript.emptySource":
      "Naciśnij „Zacznij słuchać” i włącz coś, w czym ktoś mówi - albo jeden z przykładów w ustawieniach. To, co zostanie powiedziane, pojawi się tutaj.",
    "transcript.emptyTarget":
      "Tutaj pojawi się tłumaczenie - fragment po fragmencie, jeszcze zanim mówiący skończy.",
    "transcript.translating": "tłumaczenie…",
    "transcript.notTranslated": "- nieprzetłumaczone",
    "transcript.toBottom": "Podążaj za ostatnim wierszem",

    "settings.general": "Ogólne",
    "settings.models": "Modele",
    "settings.about": "O programie",
    "settings.audioTitle": "Dźwięk",
    "settings.audioNote":
      "Skąd bierze się dźwięk. Wszystko, co odtwarza ten Mac, albo jedna aplikacja.",
    "settings.audio": "Źródło dźwięku",
    "settings.recognition": "Rozpoznawanie mowy",
    "settings.recognitionNote":
      "Zamienia to, co gra na komputerze, w tekst. Potrzebny jest model rozpoznawania.",
    "settings.translation": "Tłumaczenie",
    "settings.translationNote":
      "Przekłada rozpoznany tekst na inny język. Potrzebny jest model tłumaczenia - inny niż ten powyżej.",
    "settings.on": "Wł.",
    "settings.showOriginal": "Pokazuj oryginał obok tłumaczenia",
    "settings.locked": "Zatrzymaj słuchanie, aby zmienić te ustawienia.",
    "settings.noModel": "Nie zainstalowano modelu",
    "settings.needVad":
      "Zainstaluj poniżej model wykrywania mowy - bez niego rozpoznawanie nie zadziała.",
    "settings.needAsr": "Zainstaluj poniżej model rozpoznawania.",
    "settings.needMt": "Zainstaluj poniżej model tłumaczenia.",
    "settings.detect": "Wykryj automatycznie",
    "settings.model": "Model",
    "settings.interface": "Interfejs",
    "settings.interfaceNote":
      "Jak wygląda samo okno. Nic z tego nie zmienia tego, co aplikacja robi z dźwiękiem.",
    "settings.language": "Język",
    "settings.theme": "Motyw",
    "settings.textSize": "Wielkość tekstu",
    "settings.samples": "Wypróbuj bez szukania filmu",
    "settings.samplesNote":
      "Odtwarza nagrany fragment przez głośniki, więc idzie tą samą drogą co każdy inny dźwięk. Najpierw włącz słuchanie.",

    "theme.dark": "Ciemny",
    "theme.light": "Jasny",
    "theme.system": "Jak w systemie",

    "about.title": "O programie",
    "about.tagline":
      "Napisy na żywo i tłumaczenie wszystkiego, co gra na tym komputerze. Rozpoznawanie i tłumaczenie działają tutaj - bez konta, bez klucza API i bez wysyłania czegokolwiek na zewnątrz.",
    "about.version": "Wersja",
    "about.runtime": "Środowisko",
    "about.license": "Licencja",
    "about.source": "Kod źródłowy",
    "about.issues": "Zgłoś problem",
    "about.licenseFile": "Przeczytaj licencję",
    "about.notices": "Licencje innych autorów",
    "about.built": "Zbudowano z użyciem",
    "about.how": "Jak to działa",
    "about.howCapture":
      "Dźwięk pochodzi wprost z macOS, przez podsłuch procesu w Core Audio: dokładnie to, co ten Mac już odtwarza - wszystko naraz albo jedna aplikacja. Bez wirtualnego sterownika audio, bez mikrofonu i bez żadnej zmiany w tym, co słyszysz.",
    "about.howPipeline":
      "Ten strumień jest miksowany do mono w 16 kHz i zostaje w pamięci. Silero VAD tnie go na zdania, whisper.cpp zamienia je w tekst na GPU, a llama.cpp tłumaczy w osobnym procesie, gdy mówiący jeszcze mówi. Sam dźwięk nigdy nie trafia na dysk i nigdy nie opuszcza tego Maca.",

    "models.title": "Modele",
    "models.note":
      "Wszystko działa na tym komputerze, więc każdy model to plik na tym dysku. Zainstaluj jeden do rozpoznawania i jeden do tłumaczenia.",
    "models.onDisk": "na dysku",
    "models.forRecognition": "rozpoznawanie",
    "models.forTranslation": "tłumaczenie",
    "models.forVad": "wykrywanie mowy",
    "models.recommended": "zalecany",
    "models.required": "wymagany",
    "models.installed": "zainstalowany",
    "models.of": "z",

    "history.empty":
      "Na razie pusto. Za każdym razem, gdy zaczynasz słuchać, to, co zostało rozpoznane i przetłumaczone, trafia tutaj.",
    "history.pick": "Wybierz sesję po lewej.",
    "history.loading": "Wczytywanie…",
    "history.rows": "wierszy",
    "history.words": "słów",
    "history.exportText": "Eksportuj tekst",
    "history.exportSrt": "Eksportuj napisy",
    "history.exportJson": "Eksportuj JSON",
    "history.saved": "Zapisano w",
    "history.noModel": "brak modelu rozpoznawania",
    "history.notTranslated": "nieprzetłumaczone",
    "history.segment": "fragment",
    "history.segments": "fragmentów",

    "size.small": "Mały",
    "size.medium": "Średni",
    "size.large": "Duży",
    "size.huge": "Bardzo duży",
  },
  phrases: {
    idle: [
      "Cześć. Przetłumaczymy coś?",
      "Włącz cokolwiek, w czym ktoś mówi",
      "Gotowy, kiedy tylko zechcesz",
      "Cicho tu. I tak słucham",
      "Jedno kliknięcie i zaczną płynąć słowa",
      "Nic z tego nie opuszcza tego Maca",
      "Co dziś oglądamy?",
      "Podcast, wykład, rozmowa - mnie wszystko jedno",
      "Czekam na pierwsze zdanie",
      "Włącz słuchanie i ruszamy",
    ],
    live: [
      "Słucham uważnie…",
      "Próbuję rozróżnić słowa…",
      "Łapię zdania w locie…",
      "To na pewno było słowo…",
      "Kartkuję swój słownik…",
      "Nadążam, mniej więcej…",
      "Tłumaczę szybciej, niż myślę…",
      "Zamieniam się w słuch…",
      "Zaraz będę miał to zdanie…",
      "Trzymam się wątku…",
    ],
  },
};

export default pl;
