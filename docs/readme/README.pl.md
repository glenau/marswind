<div align="center">

<img src="../../src-tauri/icons/128x128@2x.png" alt="" width="128" height="128">

# Marswind

**Napisy i tłumaczenie na żywo dla dźwięku z twojego komputera. W pełni
offline.**

[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](../../LICENSE)
[![Platform: macOS 14.4+](https://img.shields.io/badge/platform-macOS%2014.4%2B-lightgrey.svg)](#platformy)
[![Version](https://img.shields.io/badge/version-0.1.1-brightgreen.svg)](#)

[English](../../README.md) ·
[Русский](README.ru.md) ·
[Deutsch](README.de.md) ·
[Español](README.es.md) ·
[Français](README.fr.md) ·
[Italiano](README.it.md) ·
[Português](README.pt.md) ·
**Polski** ·
[Türkçe](README.tr.md) ·
[Українська](README.uk.md) ·
[中文](README.zh.md) ·
[日本語](README.ja.md) ·
[한국어](README.ko.md)

<img src="../screenshot.png" alt="Okno Marswind: oryginał po lewej, tłumaczenie na hiszpański po prawej" width="900">

</div>

Marswind słucha tego, co odtwarza twój komputer - filmu na YouTube, rozmowy w
Google Meet, Teams czy Zoomie, lokalnego pliku wideo - rozpoznaje mowę i
tłumaczy ją na wybrany język w trakcie mówienia.

Bez kluczy API, bez kont, bez internetu. Modele pobiera się raz, a potem
działają lokalnie; dźwięk zostaje w pamięci, nigdy nie trafia na dysk ani
nigdzie indziej.

## Co potrafi

- **Przechwytuje dźwięk systemowy** bez wirtualnego sterownika audio - wszystko,
  co odtwarza komputer, albo jedną aplikację, na przykład przeglądarkę
- **Rozpoznaje mowę** przez whisper.cpp na GPU: napisy rosną w miarę mówienia,
  zamiast być przepisywane pod czytającym
- **Tłumaczy w trakcie mówienia** - słowa trafiają do tłumacza od razu po
  ustaleniu, a nie po zakończeniu zdania, a tłumaczenie przychodzi słowo po słowie
- **Zarządza modelami** z poziomu aplikacji: sześć modeli rozpoznawania i trzy
  tłumaczenia, wszystkie na MIT albo Apache-2.0, pobierane z postępem i
  weryfikacją SHA-256
- **Zapisuje każdą sesję** - można je przeglądać i eksportować jako tekst, napisy
  (`.srt`) albo JSON wraz z czasami
- **Zawiera przykładowe nagrania**, żeby dało się to sprawdzić bez szukania filmu
- **Mówi w trzynastu językach** - tych samych, na które tłumaczy - w jasnym lub
  ciemnym motywie, z rozmiarem tekstu skalującym cały interfejs, a nie tylko czcionkę

### Języki

Angielski, rosyjski, niemiecki, hiszpański, francuski, włoski, portugalski,
polski, turecki, ukraiński, chiński, japoński i koreański - zarówno jako języki
docelowe, jak i język samego okna. Język mówiony jest domyślnie rozpoznawany z
dźwięku, a rozpoznawanie obejmuje wszystko, co obsługuje whisper.

## Jak to działa

```
Dźwięk systemowy  →  przepróbkowanie do 16 kHz mono  →  detekcja mowy (Silero)
                  →  rozpoznawanie mowy (whisper.cpp)
                  →  tłumaczenie (llama.cpp, w osobnym procesie)
                  →  transkrypcja: oryginał po lewej, tłumaczenie obok
```

Wszystko poniżej interfejsu napisane jest w Ruście i działa na dedykowanych
wątkach, a tłumaczenie mieszka w osobnym pliku wykonywalnym, bo whisper.cpp i
llama.cpp nie mogą dzielić jednego procesu. Projekt i jego uzasadnienia są w
[docs/ARCHITECTURE.md](../ARCHITECTURE.md).

Zmierzone na Apple Silicon z domyślnymi modelami, na syntetycznym korpusie z
[tests/](../../tests/README.md) - mediany z trzech przebiegów na klip: pierwszy
napis mniej więcej 6 sekund od początku, kolejny co 2-3 sekundy, a odsetek
błędów na słowo od 4 % przy czystym czytaniu do 23 % przy klipie pełnym nazw
własnych i liczb. Rozpoznawanie nie jest deterministyczne, a pojedynczy przebieg
waha się o jakieś dwadzieścia punktów - to więc mediany, a nie wyniki; skąd
biorą się te liczby, opisano obok samego stanowiska testowego.

## Platformy

| Platforma | Stan |
|---|---|
| **macOS 14.4+** | Obsługiwana - Core Audio process taps, Metal |
| **Windows** | W trakcie prac - WASAPI loopback |
| **Linux** | W trakcie prac - PipeWire |

Aplikacja już się kompiluje i uruchamia na Windowsie i Linuksie, ale przechwyt
dźwięku zgłasza tam swoją niedostępność - czyli okno bez niczego do słuchania.
Wszystko powyżej przechwytu jest już niezależne od platformy.

Wirtualny sterownik audio, taki jak BlackHole, **nie** jest potrzebny na żadnej
platformie: przechwyt idzie przez natywne API systemu.

## Wymagania

| | |
|---|---|
| macOS | 14.4 lub nowszy, Apple Silicon albo Intel |
| Pamięć | 8 GB dla samego rozpoznawania, 16 GB z tłumaczeniem |
| Dysk | 0,1-6,5 GB na wybrane modele |
| Do zbudowania | [Rust](https://rustup.rs), [Node.js](https://nodejs.org) 20+, cmake (`brew install cmake`) |

## Instalacja

### Pobierz

[Najnowsze wydanie](https://github.com/glenau/marswind/releases/latest) zawiera
plik `.dmg`. Otwórz go, przeciągnij Marswind do Aplikacji i gotowe - około 13
MB, bo modele pobierane są później i tylko te, które wybierzesz.

**macOS odmówi otwarcia go za pierwszym razem.** Obraz jest podpisany, ale nie
notaryzowany: za projektem nie stoi płatny certyfikat Developer ID, a wszystko
bez niego Gatekeeper traktuje jako nieznane. Obejście:

1. Otwórz aplikację i pozwól ją zablokować. Naciśnij **Done**, nie
   "Move to Bin".
2. **Ustawienia systemowe → Prywatność i ochrona**, przewiń do sekcji
   **Ochrona**. Będzie tam informacja, że Marswind został zablokowany, a obok
   przycisk **Otwórz mimo to**.
3. Naciśnij go, potwierdź Touch ID lub hasłem i potwierdź jeszcze raz.

macOS zapyta raz i zapamięta. Przycisk pojawia się dopiero po zablokowanym
uruchomieniu i żyje około godziny; jeśli go nie ma, otwórz aplikację ponownie.

Prawy przycisk na aplikacji → Otwórz to wcześniejszy skrót do tego samego i
nadal działa na macOS 14. macOS 15 go usunął, więc droga przez ustawienia jest
tą, która działa wszędzie.

### Albo zbuduj

```bash
git clone https://github.com/glenau/marswind.git
cd marswind
npm install
npm run install:macos
```

To zbuduje worker tłumaczenia, zbuduje pakiet release, podpisze go podpisem
ad-hoc i skopiuje do `/Applications/Marswind.app`. Pierwsza kompilacja trwa
kilka minut - whisper.cpp i llama.cpp kompilują się ze źródeł. Nic więcej nie
jest potrzebne: żadnych podmodułów do pobrania, żadnych bibliotek do ręcznej
instalacji i żadnych modeli do wcześniejszego ściągnięcia.

### Pierwsze uruchomienie

1. `open /Applications/Marswind.app`
2. macOS poprosi o uprawnienie **Nagrywanie dźwięku**. Zgódź się - bez niego
   aplikacja nic nie słyszy. Jeśli odmówiono, można je przyznać ponownie w
   Ustawieniach systemowych → Prywatność i ochrona → Nagrywanie dźwięku.
3. Otwórz **Ustawienia** i pobierz jeden model rozpoznawania i jeden model
   tłumaczenia. `Large v3 Turbo (compressed)` i `Qwen3 4B Instruct` to domyślne
   wybory od 16 GB w górę; `Small` i `Qwen3 1.7B` mieszczą się w 8 GB. Około 3 GB
   pobierania, sprawdzanego wobec opublikowanej sumy kontrolnej. Przy każdym
   modelu podana jest jego licencja - zobacz [docs/MODELS.md](../MODELS.md).
4. Naciśnij **Zacznij słuchać** i włącz coś z mową. W ustawieniach są cztery
   przykładowe nagrania, jeśli wolisz nie szukać filmu.

Dwie rzeczy o kopii zbudowanej samodzielnie:

- **Jest podpisana ad-hoc.** Podpis jest stały dla danej kompilacji, więc
  uprawnienie do dźwięku się utrzymuje - ale przebudowa tworzy nową tożsamość i
  macOS pyta ponownie. Kończy to certyfikat Developer ID, którego jeszcze nie ma.
- **Nie przenoś aplikacji w trakcie działania.** Aby ją zaktualizować, uruchom
  ponownie `npm run install:macos`; podmienia `/Applications/Marswind.app` w miejscu.

### Aktualizacja

**Ustawienia → O programie → Sprawdź aktualizacje.** Aplikacja pyta GitHuba, czy
jest nowsze wydanie; jeśli jest, pobiera obraz do Pobranych, porównuje go z sumą
kontrolną opublikowaną obok i pokazuje w Finderze. Instalacja to to samo
przeciągnięcie co za pierwszym razem.

Nic nie sprawdza się samo: żadnego timera ani sprawdzania przy starcie, bo
aplikacja nie wykonuje żądań sieciowych, których nie nacisnąłeś.

Kopię zbudowaną samodzielnie aktualizuje się tak, jak ją zainstalowano: ponownie
`npm run install:macos`.

### Budowanie obrazu dysku

```bash
npm run build:dmg
```

Buduje pakiet wydania, podpisuje go i pakuje do
`src-tauri/target/Marswind-<wersja>-<arch>.dmg` - ten sam obraz, który dołącza
się do wydania, z tym samym zastrzeżeniem o Gatekeeperze co wyżej. Lista
kontrolna wokół tego jest w [docs/RELEASING.md](../RELEASING.md).

## Rozwój

`tauri dev` tworzy goły plik wykonywalny bez `Info.plist` i bez podpisu, a
process tapy Core Audio w takiej postaci nie działają. Zamiast tego użyj
polecenia, które buduje pakiet debugowy, podpisuje go i uruchamia:

```bash
npm run dev:macos
```

| Polecenie | Co robi |
|---|---|
| `npm run dev:macos` | zbuduj, podpisz i uruchom pakiet debugowy |
| `npm run install:macos` | zbuduj pakiet release i zainstaluj go |
| `npm run check` | typy Svelte i TypeScript |
| `npm run build:dmg` | podpisany `.dmg` do przekazania komuś |
| `npm run build:sidecar` | sam worker tłumaczenia |
| `npm run build:icons` | przerysuj ikonę z `scripts/make-icon.py` |
| `npm run build:social` | przerysować kartę, którą GitHub pokazuje przy linku |
| `npm run licenses` | wygenerować `THIRD-PARTY-NOTICES.md` z plików lock |

Nie ma CI: whisper.cpp i llama.cpp kompilują się ze źródeł, a stanowisko testowe
odtwarza dźwięk przez wyjście systemowe, więc każda kontrola to lokalne
polecenie. [CONTRIBUTING.md](../../CONTRIBUTING.md) je wylicza.

## Testy

Testy jednostkowe pokrywają czystą logikę; skrypty w
[tests/](../../tests/README.md) odtwarzają dźwięk przez wyjście systemowe i
oceniają to, co wychodzi z prawdziwego potoku - rozpoznawanie, tłumaczenie i
opóźnienia razem.

```bash
npm run build:sidecar
cargo test --manifest-path src-tauri/Cargo.toml
```

Pierwsza linia potrzebna jest raz, a potem tylko po `cargo clean`. Tauri pakuje
worker tłumaczenia jako sidecar, więc jego skrypt budowania w ogóle odmawia
zbudowania `src-tauri`, dopóki binarki nie ma: na świeżym klonie samo
`cargo test` zatrzymuje się na
`resource path 'binaries/marswind-translator-…' doesn't exist`. Każde polecenie
`npm run` wykonuje ten krok za ciebie; bezpośrednie wywołanie `cargo` nie.

Skrypty potoku potrzebują zbudowanego i podpisanego pakietu oraz zainstalowanych
modeli:

```bash
npm run dev:macos
tests/run-capture.sh
tests/run-asr.sh
tests/run-pipeline.sh
```

Pojedynczy przebieg po korpusie waha się o około dwadzieścia punktów błędu na
słowach, więc jedna liczba sama w sobie nic nie znaczy. Porównuj mediany z wielu
przebiegów i czytaj transkrypcje, a nie tylko wyniki.

## Prywatność

- Dźwięk jest przechwytywany, przepróbkowywany i rozpoznawany **w pamięci**. Nigdy
  nie trafia na dysk ani nigdzie indziej.
- Jedyny ruch sieciowy to ten, dla którego naciskasz przycisk: pobranie modelu
  albo sprawdzenie aktualizacji. Nic nie dzieje się z timera ani przy starcie.
- Bez telemetrii, bez analityki, bez raportów awarii, bez konta.
- Transkrypcje zapisywane są wyłącznie w katalogu danych aplikacji, żeby widok
  Historii miał co pokazać. Usuwa się je z poziomu aplikacji.

## Współtworzenie

Zgłoszenia błędów, pomysły i pull requesty są mile widziane.
[CONTRIBUTING.md](../../CONTRIBUTING.md) opisuje konfigurację, kontrole,
konwencję commitów i to, na co patrzy przegląd. Przed czymś większym otwórz
issue - kilka oczywistych usprawnień już próbowano i wycofano, z zapisanymi
pomiarami.

- [Kodeks postępowania](../../CODE_OF_CONDUCT.md)
- [Polityka bezpieczeństwa](../../SECURITY.md) - podatności zgłaszaj prywatnie, nie
  w issue

## Zbudowane na

| | | |
|---|---|---|
| [whisper.cpp](https://github.com/ggml-org/whisper.cpp) | MIT | rozpoznawanie, a wraz z nim implementacja Silero VAD |
| [llama.cpp](https://github.com/ggml-org/llama.cpp) | MIT | tłumaczenie, we własnym procesie |
| [ggml](https://github.com/ggml-org/ggml) | MIT | biblioteka tensorów i backend Metal pod obydwoma |
| [whisper-rs](https://codeberg.org/tazz4843/whisper-rs) | Unlicense | wiązanie Rusta do whisper.cpp |
| [llama-cpp-2](https://github.com/utilityai/llama-cpp-rs) | MIT / Apache-2.0 | wiązanie Rusta do llama.cpp |
| [Silero VAD](https://github.com/snakers4/silero-vad) | MIT | model znajdujący granice fraz |
| [Tauri](https://tauri.app) | MIT / Apache-2.0 | okno i granica procesów |
| [Svelte](https://svelte.dev) | MIT | interfejs |
| [rubato](https://github.com/HEnquist/rubato) | MIT | resampler FFT przed whisperem |

Całe drzewo zależności, z licencją każdego pakietu, jest w
[THIRD-PARTY-NOTICES.md](../../THIRD-PARTY-NOTICES.md) - generowanym z plików
lock i dołączanym do aplikacji obok samej licencji.

**Modeli to wszystko nie obejmuje.** Pobierane są z
[Hugging Face](https://huggingface.co) na twoją prośbę i zachowują warunki swoich
wydawców - a do katalogu trafiają tylko te, których warunki można przyjąć bez
czytania: modele whisper i Silero są na MIT, Qwen3 na Apache-2.0. Każdy wiersz w
ustawieniach podaje swoją licencję, zanim ruszy pobieranie. Szczegóły w
[docs/MODELS.md](../MODELS.md).

## Licencja

MIT - zobacz [LICENSE](../../LICENSE). To dotyczy tego repozytorium; nie dotyczy
modeli, a powyższe informacje nie zastępują licencji, na które wskazują.
