import type { Dictionary } from "./en";

const fr: Dictionary = {
  strings: {
    "stage.audio": "Audio",
    "stage.recognition": "Reconnaissance",
    "stage.translation": "Traduction",
    "stage.notCapturing": "Aucune capture",
    "stage.notRunning": "À l'arrêt",
    "meter.level": "Niveau d'entrée",
    "action.start": "Commencer à écouter",
    "action.stop": "Arrêter",
    "action.working": "Démarrage…",
    "action.history": "Historique",
    "action.settings": "Réglages",
    "action.clear": "Effacer",
    "action.dismiss": "Masquer",
    "action.refresh": "Actualiser",
    "action.delete": "Supprimer",
    "action.play": "Écouter",
    "action.text": "Texte",
    "action.hideText": "Masquer le texte",
    "action.install": "Installer",
    "action.remove": "Retirer",
    "action.cancel": "Annuler",
    "action.github": "Le projet sur GitHub",

    "transcript.original": "Original",
    "transcript.translation": "Traduction",
    "transcript.into": "Traduit en",
    "transcript.toLanguage": "en",
    "transcript.emptySource":
      "Appuyez sur « Commencer à écouter », puis lancez quelque chose où l'on parle - ou l'un des extraits des réglages. Ce qui est dit apparaîtra ici.",
    "transcript.emptyTarget":
      "La traduction apparaît ici, bout de phrase après bout de phrase, pendant que la personne parle encore.",
    "transcript.translating": "traduction…",
    "transcript.notTranslated": "- non traduit",
    "transcript.toBottom": "Suivre la dernière ligne",

    "settings.general": "Général",
    "settings.models": "Modèles",
    "settings.about": "À propos",
    "settings.audioTitle": "Audio",
    "settings.audioNote":
      "D'où vient le son. Tout ce que joue ce Mac, ou une seule application.",
    "settings.audio": "Source audio",
    "settings.recognition": "Reconnaissance vocale",
    "settings.recognitionNote":
      "Transforme en texte ce que la machine est en train de jouer. Nécessite un modèle de reconnaissance.",
    "settings.translation": "Traduction",
    "settings.translationNote":
      "Fait passer le texte reconnu dans une autre langue. Nécessite un modèle de traduction - un autre que celui du dessus.",
    "settings.on": "Activée",
    "settings.showOriginal": "Afficher l'original à côté de la traduction",
    "settings.locked": "Arrêtez l'écoute pour modifier ces réglages.",
    "settings.noModel": "Aucun modèle installé",
    "settings.needVad":
      "Installez ci-dessous le modèle de détection de la voix - la reconnaissance en a besoin.",
    "settings.needAsr": "Installez un modèle de reconnaissance ci-dessous.",
    "settings.needMt": "Installez un modèle de traduction ci-dessous.",
    "settings.detect": "Détecter automatiquement",
    "settings.model": "Modèle",
    "settings.interface": "Interface",
    "settings.interfaceNote":
      "L'aspect de la fenêtre. Rien de tout cela ne change ce que l'application fait du son.",
    "settings.language": "Langue",
    "settings.theme": "Thème",
    "settings.textSize": "Taille du texte",
    "settings.samples": "Essayer sans chercher de vidéo",
    "settings.samplesNote":
      "Joue un extrait enregistré dans les haut-parleurs : il emprunte donc le même chemin que n'importe quel autre son. Commencez par lancer l'écoute.",

    "theme.dark": "Sombre",
    "theme.light": "Clair",
    "theme.system": "Comme le système",

    "about.title": "À propos",
    "about.tagline":
      "Sous-titres et traduction en direct de tout ce que joue cet ordinateur. La reconnaissance et la traduction tournent ici - sans compte, sans clé d'API, et rien ne quitte la machine.",
    "about.version": "Version",
    "about.runtime": "Environnement",
    "about.license": "Licence",
    "about.source": "Code source",
    "about.issues": "Signaler un problème",
    "about.licenseFile": "Lire la licence",
    "about.notices": "Licences tierces",
    "about.built": "Construit avec",
    "about.how": "Comment ça marche",
    "about.howCapture":
      "Le son vient de macOS elle-même, par une prise de processus Core Audio : exactement ce que votre Mac joue déjà - tout à la fois ou une seule application. Aucun pilote audio virtuel, aucun micro, et rien ne change à ce que vous entendez.",
    "about.howPipeline":
      "Ce flux est mixé en mono à 16 kHz et reste en mémoire. Silero VAD le découpe en phrases, whisper.cpp les transforme en texte sur le GPU, et llama.cpp traduit dans un processus séparé pendant que la personne parle encore. L'audio lui-même n'est jamais écrit sur le disque et ne quitte jamais ce Mac.",

    "models.title": "Modèles",
    "models.note":
      "Tout tourne sur cette machine, donc chaque modèle est un fichier sur ce disque. Installez-en un pour la reconnaissance et un pour la traduction.",
    "models.onDisk": "sur le disque",
    "models.forRecognition": "reconnaissance",
    "models.forTranslation": "traduction",
    "models.forVad": "détection de la voix",
    "models.recommended": "recommandé",
    "models.required": "obligatoire",
    "models.installed": "installé",
    "models.of": "sur",

    "history.empty":
      "Aucune session pour l'instant. Chaque fois que vous lancez l'écoute, ce qui est reconnu et traduit est écrit ici.",
    "history.pick": "Choisissez une session à gauche.",
    "history.loading": "Chargement…",
    "history.rows": "lignes",
    "history.words": "mots",
    "history.exportText": "Exporter le texte",
    "history.exportSrt": "Exporter les sous-titres",
    "history.exportJson": "Exporter en JSON",
    "history.saved": "Enregistré dans",
    "history.noModel": "aucun modèle de reconnaissance",
    "history.notTranslated": "non traduit",
    "history.segment": "segment",
    "history.segments": "segments",

    "size.small": "Petit",
    "size.medium": "Moyen",
    "size.large": "Grand",
    "size.huge": "Très grand",
  },
  phrases: {
    idle: [
      "Bonjour. On traduit quelque chose ?",
      "Lancez n'importe quoi où l'on parle",
      "Prêt quand vous voulez",
      "C'est calme ici. J'écoute quand même",
      "Un clic et les mots arrivent",
      "Rien de ce que vous jouez ne quitte ce Mac",
      "On regarde quoi aujourd'hui ?",
      "Un podcast, un cours, un appel - tout me va",
      "J'attends la première phrase",
      "Lancez l'écoute et c'est parti",
    ],
    live: [
      "J'écoute attentivement…",
      "J'essaie de distinguer les mots…",
      "J'attrape les phrases au vol…",
      "Ça, c'était un mot, sûr…",
      "Je feuillette mon dictionnaire…",
      "Je suis le rythme, à peu près…",
      "Je traduis plus vite que je ne pense…",
      "Tout ouïe, sans distraction…",
      "Je tiens presque cette phrase…",
      "Je suis le fil…",
    ],
  },
};

export default fr;
