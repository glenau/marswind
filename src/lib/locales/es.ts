import type { Dictionary } from "./en";

const es: Dictionary = {
  strings: {
    "stage.audio": "Audio",
    "stage.recognition": "Reconocimiento",
    "stage.translation": "Traducción",
    "stage.notCapturing": "Sin captura",
    "stage.notRunning": "Detenido",
    "meter.level": "Nivel de entrada",
    "action.start": "Empezar a escuchar",
    "action.stop": "Parar",
    "action.working": "Iniciando…",
    "action.history": "Historial",
    "action.settings": "Ajustes",
    "action.clear": "Limpiar",
    "action.dismiss": "Ocultar",
    "action.refresh": "Actualizar",
    "action.delete": "Eliminar",
    "action.play": "Reproducir",
    "action.text": "Texto",
    "action.hideText": "Ocultar el texto",
    "action.install": "Instalar",
    "action.remove": "Quitar",
    "action.cancel": "Cancelar",
    "action.github": "El proyecto en GitHub",

    "transcript.original": "Original",
    "transcript.translation": "Traducción",
    "transcript.into": "Traducido al",
    "transcript.toLanguage": "al",
    "transcript.emptySource":
      "Pulse «Empezar a escuchar» y reproduzca algo hablado - o uno de los ejemplos de los ajustes. Aquí aparecerá lo que se diga.",
    "transcript.emptyTarget":
      "Aquí aparece la traducción, frase a frase, mientras la persona sigue hablando.",
    "transcript.translating": "traduciendo…",
    "transcript.notTranslated": "- sin traducir",
    "transcript.toBottom": "Seguir la última línea",

    "settings.general": "General",
    "settings.models": "Modelos",
    "settings.about": "Acerca de",
    "settings.audioTitle": "Audio",
    "settings.audioNote":
      "De dónde sale el sonido. Todo lo que reproduce este Mac, o una sola aplicación.",
    "settings.audio": "Fuente de audio",
    "settings.recognition": "Reconocimiento de voz",
    "settings.recognitionNote":
      "Convierte en texto lo que está sonando. Necesita un modelo de reconocimiento.",
    "settings.translation": "Traducción",
    "settings.translationNote":
      "Pasa el texto reconocido a otro idioma. Necesita un modelo de traducción, distinto del de arriba.",
    "settings.on": "Activada",
    "settings.showOriginal": "Mostrar el original junto a la traducción",
    "settings.locked": "Deje de escuchar para cambiar esto.",
    "settings.noModel": "Ningún modelo instalado",
    "settings.needVad":
      "Instale abajo el modelo de detección de voz: el reconocimiento lo necesita.",
    "settings.needAsr": "Instale abajo un modelo de reconocimiento.",
    "settings.needMt": "Instale abajo un modelo de traducción.",
    "settings.detect": "Detectar automáticamente",
    "settings.model": "Modelo",
    "settings.interface": "Interfaz",
    "settings.interfaceNote":
      "El aspecto de la ventana. Nada de esto cambia lo que la aplicación hace con el sonido.",
    "settings.language": "Idioma",
    "settings.theme": "Tema",
    "settings.textSize": "Tamaño del texto",
    "settings.samples": "Probarlo sin buscar un vídeo",
    "settings.samplesNote":
      "Reproduce un fragmento grabado por los altavoces, así que recorre el mismo camino que cualquier otro sonido. Empiece a escuchar antes.",

    "theme.dark": "Oscuro",
    "theme.light": "Claro",
    "theme.system": "Según el sistema",

    "about.title": "Acerca de",
    "about.tagline":
      "Subtítulos y traducción en directo de lo que suene en este ordenador. El reconocimiento y la traducción se ejecutan aquí: sin cuenta, sin clave de API y sin que nada salga de la máquina.",
    "about.version": "Versión",
    "about.runtime": "Entorno",
    "about.license": "Licencia",
    "about.source": "Código fuente",
    "about.issues": "Informar de un problema",
    "about.licenseFile": "Leer la licencia",
    "about.notices": "Licencias de terceros",
    "about.built": "Hecho con",
    "about.how": "Cómo funciona",
    "about.howCapture":
      "El sonido se toma de la propia macOS, mediante una derivación de proceso de Core Audio: lo que este Mac ya está reproduciendo, todo a la vez o una sola aplicación. Sin controlador de audio virtual, sin micrófono, y sin cambiar nada de lo que usted oye.",
    "about.howPipeline":
      "Ese flujo se mezcla a mono a 16 kHz y se queda en memoria. Silero VAD lo corta en frases, whisper.cpp las convierte en texto en la GPU y llama.cpp traduce en un proceso aparte mientras la persona sigue hablando. El audio en sí nunca se escribe en el disco ni sale de este Mac.",

    "models.title": "Modelos",
    "models.note":
      "Todo se ejecuta en esta máquina, así que cada modelo es un archivo en este disco. Instale uno para reconocer y otro para traducir.",
    "models.onDisk": "en el disco",
    "models.forRecognition": "reconocimiento",
    "models.forTranslation": "traducción",
    "models.forVad": "detección de voz",
    "models.recommended": "recomendado",
    "models.required": "obligatorio",
    "models.installed": "instalado",
    "models.of": "de",

    "history.empty":
      "Todavía no hay sesiones. Cada vez que pulse «Empezar a escuchar», aquí se guardará lo reconocido y lo traducido.",
    "history.pick": "Elija una sesión a la izquierda.",
    "history.loading": "Cargando…",
    "history.rows": "líneas",
    "history.words": "palabras",
    "history.exportText": "Exportar texto",
    "history.exportSrt": "Exportar subtítulos",
    "history.exportJson": "Exportar JSON",
    "history.saved": "Guardado en",
    "history.noModel": "sin modelo de reconocimiento",
    "history.notTranslated": "sin traducir",
    "history.segment": "segmento",
    "history.segments": "segmentos",

    "size.small": "Pequeño",
    "size.medium": "Mediano",
    "size.large": "Grande",
    "size.huge": "Muy grande",
  },
  phrases: {
    idle: [
      "Hola. ¿Traducimos algo?",
      "Ponga cualquier cosa en la que se hable",
      "Listo cuando usted lo esté",
      "Qué silencio. Yo sigo escuchando",
      "Un clic y empiezan a llegar las palabras",
      "Nada de lo que ponga sale de este Mac",
      "¿Qué vemos hoy?",
      "Un pódcast, una clase, una llamada: me da igual",
      "Esperando la primera frase",
      "Pulse «Empezar a escuchar» y en marcha",
    ],
    live: [
      "Escuchando con atención…",
      "Intentando distinguir las palabras…",
      "Cazando frases al vuelo…",
      "Eso ha sido una palabra, seguro…",
      "Hojeando mi diccionario…",
      "Siguiendo el ritmo, más o menos…",
      "Traduzco más rápido de lo que pienso…",
      "Todo oídos, sin distracciones…",
      "Ya casi tengo esa frase…",
      "Sigo el hilo…",
    ],
  },
};

export default es;
