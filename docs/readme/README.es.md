<div align="center">

<img src="../../src-tauri/icons/128x128@2x.png" alt="" width="128" height="128">

# Marswind

**Subtítulos y traducción en vivo del audio de tu ordenador. Totalmente sin
conexión.**

[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](../../LICENSE)
[![Platform: macOS 14.4+](https://img.shields.io/badge/platform-macOS%2014.4%2B-lightgrey.svg)](#plataformas)
[![Version](https://img.shields.io/badge/version-0.1.1-brightgreen.svg)](#)

[English](../../README.md) ·
[Русский](README.ru.md) ·
[Deutsch](README.de.md) ·
**Español** ·
[Français](README.fr.md) ·
[Italiano](README.it.md) ·
[Português](README.pt.md) ·
[Polski](README.pl.md) ·
[Türkçe](README.tr.md) ·
[Українська](README.uk.md) ·
[中文](README.zh.md) ·
[日本語](README.ja.md) ·
[한국어](README.ko.md)

<img src="../screenshot.png" alt="La ventana de Marswind: el original a la izquierda, su traducción al español a la derecha" width="900">

</div>

Marswind escucha lo que suena en tu máquina - un vídeo de YouTube, una llamada
de Google Meet, Teams o Zoom, un archivo de vídeo local - reconoce el habla y la
traduce al idioma que elijas mientras la persona habla.

Sin claves de API, sin cuentas, sin internet. Los modelos se descargan una vez y
luego se ejecutan en local; el audio se queda en memoria y nunca se escribe en
disco ni se envía a ninguna parte.

## Qué hace

- **Captura el audio del sistema** sin driver de audio virtual - todo lo que
  reproduce la máquina, o una sola aplicación como el navegador
- **Reconoce el habla** con whisper.cpp en la GPU: los subtítulos crecen según se
  pronuncian, en lugar de reescribirse bajo quien lee
- **Traduce mientras se habla** - las palabras van al traductor en cuanto quedan
  fijadas, no al terminar la frase, y la traducción llega palabra a palabra
- **Gestiona los modelos** desde la propia app: seis de reconocimiento y tres de
  traducción, todos MIT o Apache-2.0, descargados con progreso y verificación
  SHA-256
- **Graba cada sesión** - se pueden consultar y exportar como texto, subtítulos
  (`.srt`) o JSON con los tiempos correspondientes
- **Incluye clips de ejemplo** para probarlo sin tener que buscar un vídeo
- **Habla trece idiomas** - los mismos a los que traduce - en tema claro u
  oscuro, con un tamaño de texto que escala toda la interfaz y no solo la letra

### Idiomas

Inglés, ruso, alemán, español, francés, italiano, portugués, polaco, turco,
ucraniano, chino, japonés y coreano, tanto como idiomas de destino como idioma
de la propia ventana. El idioma hablado se deduce del audio por defecto, y el
reconocimiento cubre todo lo que cubre whisper.

## Cómo funciona

```
Audio del sistema  →  remuestreo a 16 kHz mono  →  detección de voz (Silero)
                   →  reconocimiento del habla (whisper.cpp)
                   →  traducción (llama.cpp, en un proceso aparte)
                   →  transcripción: original a la izquierda, traducción al lado
```

Todo lo que hay bajo la interfaz está en Rust y corre en hilos dedicados, y la
traducción vive en un binario aparte porque whisper.cpp y llama.cpp no pueden
compartir proceso. El diseño y su razonamiento están en
[docs/ARCHITECTURE.md](../ARCHITECTURE.md).

Medido en Apple Silicon con los modelos por defecto, sobre el corpus sintético
de [tests/](../../tests/README.md) - medianas de tres ejecuciones por clip: el
primer subtítulo unos 6 segundos después del inicio, uno nuevo cada 2-3
segundos, y una tasa de error por palabra de entre el 4 % en una lectura limpia
y el 23 % en un clip lleno de nombres propios y cifras. El reconocimiento no es
determinista y una sola ejecución varía unos veinte puntos, así que son medianas
y no resultados; cómo se obtienen está documentado junto al banco de pruebas.

## Plataformas

| Plataforma | Estado |
|---|---|
| **macOS 14.4+** | Compatible - Core Audio process taps, Metal |
| **Windows** | En desarrollo - WASAPI loopback |
| **Linux** | En desarrollo - PipeWire |

La aplicación ya compila y arranca en Windows y Linux, pero allí la captura de
audio se declara no disponible, o sea una ventana sin nada que escuchar. Todo lo
que está por encima de la captura es independiente de la plataforma y ya
funciona.

Un driver de audio virtual como BlackHole **no** hace falta en ninguna
plataforma: la captura usa las APIs nativas del sistema.

## Requisitos

| | |
|---|---|
| macOS | 14.4 o posterior, Apple Silicon o Intel |
| Memoria | 8 GB solo para el reconocimiento, 16 GB con traducción |
| Disco | 0,1-6,5 GB para los modelos que elijas |
| Para compilar | [Rust](https://rustup.rs), [Node.js](https://nodejs.org) 20+, cmake (`brew install cmake`) |

## Instalación

### Descargarlo

La [última versión](https://github.com/glenau/marswind/releases/latest) incluye
un `.dmg`. Ábrelo, arrastra Marswind a Aplicaciones y listo - unos 13 MB, porque
los modelos se descargan después y solo los que elijas.

**macOS se negará a abrirlo la primera vez.** La imagen está firmada pero no
notarizada: no hay ningún certificado Developer ID de pago detrás de este
proyecto, y Gatekeeper trata como desconocido todo lo que no lo tenga. La forma
de pasar:

1. Abre la app y deja que la bloquee. Pulsa **Done**, no "Move to Bin".
2. **Ajustes del Sistema → Privacidad y seguridad**, baja hasta **Seguridad**.
   Habrá una línea diciendo que Marswind fue bloqueado, y junto a ella un botón
   **Abrir de todos modos**.
3. Púlsalo, autentícate con Touch ID o contraseña y confirma una vez más.

macOS pregunta una vez y lo recuerda. El botón solo aparece tras un arranque
bloqueado y dura alrededor de una hora; si no está, vuelve a abrir la app.

Hacer clic derecho en la app y elegir Abrir era el atajo anterior para esto y
sigue funcionando en macOS 14. macOS 15 lo eliminó, así que la ruta por los
ajustes es la que funciona en todas partes.

### O compilarlo

```bash
git clone https://github.com/glenau/marswind.git
cd marswind
npm install
npm run install:macos
```

Eso compila el worker de traducción, compila el bundle de release, lo firma en
modo ad-hoc y lo copia a `/Applications/Marswind.app`. La primera compilación
tarda varios minutos - whisper.cpp y llama.cpp se compilan desde el código. No
hace falta nada más: ni submódulos que clonar, ni bibliotecas que instalar a
mano, ni modelos que descargar de antemano.

### Primer arranque

1. `open /Applications/Marswind.app`
2. macOS pide el permiso de **Grabación de audio**. Acéptalo - sin él la app no
   oye nada. Si lo rechazaste, se concede de nuevo en Ajustes del Sistema →
   Privacidad y seguridad → Grabación de audio.
3. Abre **Ajustes** y descarga un modelo de reconocimiento y uno de traducción.
   `Large v3 Turbo (compressed)` y `Qwen3 4B Instruct` son los valores por
   defecto a partir de 16 GB; `Small` y `Qwen3 1.7B` caben en 8 GB. Unos 3 GB de
   descarga, verificados contra una suma de comprobación publicada. Cada fila
   indica la licencia de sus pesos - véase [docs/MODELS.md](../MODELS.md).
4. Pulsa **Empezar a escuchar** y reproduce algo con voz. En los ajustes hay
   cuatro clips de ejemplo, por si prefieres no ir a buscar un vídeo.

Dos cosas sobre una copia compilada por ti:

- **Va firmada en modo ad-hoc.** La firma es estable para una compilación dada,
  así que el permiso de audio se mantiene - pero recompilar genera una identidad
  nueva y macOS vuelve a preguntar. Lo que lo elimina es un certificado Developer
  ID, y todavía no hay ninguno.
- **No muevas la app mientras se ejecuta.** Para actualizarla, vuelve a ejecutar
  `npm run install:macos`; reemplaza `/Applications/Marswind.app` en su sitio.

### Actualizar

**Ajustes → Acerca de → Buscar actualizaciones.** Pregunta a GitHub si hay una
versión más reciente; si la hay, descarga la imagen en Descargas, la coteja con
la suma de comprobación publicada junto a ella y la muestra en el Finder.
Instalarla es el mismo arrastre que la primera vez.

Nada se comprueba solo: ni temporizador ni comprobación al arrancar, porque la
app no hace ninguna petición de red que no hayas pulsado.

Una copia compilada por ti se actualiza como se instaló: `npm run install:macos`
otra vez.

### Crear una imagen de disco

```bash
npm run build:dmg
```

Compila el paquete de release, lo firma y lo empaqueta en
`src-tauri/target/Marswind-<versión>-<arq>.dmg` - la misma imagen que se adjunta
a una publicación, con la misma advertencia sobre Gatekeeper que arriba. La
lista de comprobación está en [docs/RELEASING.md](../RELEASING.md).

## Desarrollo

`tauri dev` produce un ejecutable desnudo sin `Info.plist` y sin firma, y los
process taps de Core Audio no funcionan de esa forma. Usa esto en su lugar -
compila un bundle de depuración, lo firma y lo lanza:

```bash
npm run dev:macos
```

| Comando | Qué hace |
|---|---|
| `npm run dev:macos` | compilar, firmar y lanzar un bundle de depuración |
| `npm run install:macos` | compilar un bundle de release e instalarlo |
| `npm run check` | tipos de Svelte y TypeScript |
| `npm run build:dmg` | un `.dmg` firmado para pasar a otra persona |
| `npm run build:sidecar` | solo el worker de traducción |
| `npm run build:icons` | redibujar el icono desde `scripts/make-icon.py` |
| `npm run build:social` | redibujar la tarjeta que GitHub muestra al compartir el enlace |
| `npm run licenses` | regenerar `THIRD-PARTY-NOTICES.md` desde los lockfiles |

No hay CI: whisper.cpp y llama.cpp se compilan desde el código y el banco de
pruebas reproduce audio por la salida del sistema, así que cada comprobación es
un comando local. [CONTRIBUTING.md](../../CONTRIBUTING.md) las enumera.

## Pruebas

Las pruebas unitarias cubren la lógica pura; los scripts de
[tests/](../../tests/README.md) reproducen audio por la salida del sistema y
puntúan lo que sale del pipeline real - reconocimiento, traducción y latencia a
la vez.

```bash
npm run build:sidecar
cargo test --manifest-path src-tauri/Cargo.toml
```

La primera línea hace falta una vez, y después solo tras un `cargo clean`.
Tauri empaqueta el worker de traducción como sidecar, así que su script de
compilación se niega a construir `src-tauri` mientras el binario no exista: en un
clon recién hecho, `cargo test` por su cuenta se detiene en
`resource path 'binaries/marswind-translator-…' doesn't exist`. Cualquier orden
`npm run` hace ese paso por ti; llamar a `cargo` directamente, no.

Los scripts de la tubería necesitan un bundle compilado y firmado, y modelos
instalados:

```bash
npm run dev:macos
tests/run-capture.sh
tests/run-asr.sh
tests/run-pipeline.sh
```

Una sola ejecución sobre el corpus varía unos veinte puntos de tasa de error,
así que una cifra aislada no significa nada. Compara medianas entre ejecuciones
y lee las transcripciones, no solo las puntuaciones.

## Privacidad

- El audio se captura, remuestrea y reconoce **en memoria**. Nunca se escribe en
  disco ni se envía a ninguna parte.
- El único tráfico de red es aquello para lo que pulsas un botón: descargar un
  modelo o buscar actualizaciones. Nada ocurre por temporizador ni al arrancar.
- Sin telemetría, sin analítica, sin informes de fallos, sin cuenta.
- Las transcripciones se escriben solo en el directorio de datos de la app, para
  que la vista de historial tenga algo que mostrar. Se borran desde la propia app.

## Contribuir

Se agradecen informes de fallos, ideas y pull requests.
[CONTRIBUTING.md](../../CONTRIBUTING.md) cubre la configuración, las
comprobaciones, la convención de commits y qué mira la revisión. Abre una issue
antes de empezar algo grande - varias mejoras evidentes ya se probaron y se
revirtieron, con las mediciones anotadas.

- [Código de conducta](../../CODE_OF_CONDUCT.md)
- [Política de seguridad](../../SECURITY.md) - informa de vulnerabilidades en
  privado, no en una issue

## Construido sobre

| | | |
|---|---|---|
| [whisper.cpp](https://github.com/ggml-org/whisper.cpp) | MIT | reconocimiento, y con él la implementación de Silero VAD |
| [llama.cpp](https://github.com/ggml-org/llama.cpp) | MIT | traducción, en su propio proceso |
| [ggml](https://github.com/ggml-org/ggml) | MIT | la biblioteca de tensores y el backend Metal bajo ambos |
| [whisper-rs](https://codeberg.org/tazz4843/whisper-rs) | Unlicense | el enlace de Rust a whisper.cpp |
| [llama-cpp-2](https://github.com/utilityai/llama-cpp-rs) | MIT / Apache-2.0 | el enlace de Rust a llama.cpp |
| [Silero VAD](https://github.com/snakers4/silero-vad) | MIT | el modelo que encuentra los límites de frase |
| [Tauri](https://tauri.app) | MIT / Apache-2.0 | la ventana y la frontera entre procesos |
| [Svelte](https://svelte.dev) | MIT | la interfaz |
| [rubato](https://github.com/HEnquist/rubato) | MIT | el remuestreador FFT delante de whisper |

Todo el árbol de dependencias, con la licencia de cada paquete, está en
[THIRD-PARTY-NOTICES.md](../../THIRD-PARTY-NOTICES.md) - generado a partir de
los lockfiles y distribuido dentro de la app junto a la propia licencia.

**Nada de eso cubre los modelos.** Se descargan de
[Hugging Face](https://huggingface.co) a petición tuya y conservan los términos de
quien los publica - y al catálogo solo llegan aquellos cuyos términos se pueden
aceptar sin leerlos: los modelos whisper y Silero son MIT, Qwen3 es Apache-2.0.
Cada fila de los ajustes nombra su licencia antes de que empiece la descarga. El
detalle, en [docs/MODELS.md](../MODELS.md).

## Licencia

MIT - véase [LICENSE](../../LICENSE). Eso cubre este repositorio; no cubre los
modelos, y las notas de arriba no sustituyen a las licencias a las que apuntan.
