<div align="center">

<img src="../../src-tauri/icons/128x128@2x.png" alt="" width="128" height="128">

# Marswind

**Sous-titres et traduction en direct de l'audio de votre ordinateur.
Entièrement hors ligne.**

[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](../../LICENSE)
[![Platform: macOS 14.4+](https://img.shields.io/badge/platform-macOS%2014.4%2B-lightgrey.svg)](#plateformes)
[![Version](https://img.shields.io/badge/version-0.1.1-brightgreen.svg)](#)

[English](../../README.md) ·
[Русский](README.ru.md) ·
[Deutsch](README.de.md) ·
[Español](README.es.md) ·
**Français** ·
[Italiano](README.it.md) ·
[Português](README.pt.md) ·
[Polski](README.pl.md) ·
[Türkçe](README.tr.md) ·
[Українська](README.uk.md) ·
[中文](README.zh.md) ·
[日本語](README.ja.md) ·
[한국어](README.ko.md)

<img src="../screenshot.png" alt="La fenêtre de Marswind : l'original à gauche, sa traduction espagnole à droite" width="900">

</div>

Marswind écoute ce que joue votre machine - une vidéo YouTube, un appel Google
Meet, Teams ou Zoom, un fichier vidéo local - reconnaît la parole et la traduit
dans la langue de votre choix au fil du discours.

Pas de clés d'API, pas de comptes, pas d'internet. Les modèles sont téléchargés
une fois puis s'exécutent localement ; l'audio reste en mémoire, n'est jamais
écrit sur disque et n'est envoyé nulle part.

## Ce qu'il fait

- **Capture l'audio du système** sans pilote audio virtuel - tout ce que joue la
  machine, ou une seule application comme le navigateur
- **Reconnaît la parole** avec whisper.cpp sur le GPU : les sous-titres
  s'allongent à mesure qu'on parle, au lieu d'être réécrits sous le lecteur
- **Traduit pendant qu'on parle** - les mots partent au traducteur dès qu'ils sont
  figés, pas une fois la phrase terminée, et la traduction arrive mot à mot
- **Gère les modèles** depuis l'application : six modèles de reconnaissance et
  trois de traduction, tous sous MIT ou Apache-2.0, téléchargés avec
  progression et vérification SHA-256
- **Enregistre chaque session** - consultables ensuite et exportables en texte,
  sous-titres (`.srt`) ou JSON avec les temps correspondants
- **Fournit des extraits d'exemple**, pour l'essayer sans partir chercher une vidéo
- **Parle treize langues** - les mêmes que celles vers lesquelles il traduit - en
  thème clair ou sombre, avec une taille de texte qui met à l'échelle toute
  l'interface et pas seulement les caractères

### Langues

Anglais, russe, allemand, espagnol, français, italien, portugais, polonais,
turc, ukrainien, chinois, japonais et coréen, aussi bien comme langues cibles
que comme langue de la fenêtre. La langue parlée est déduite de l'audio par
défaut, et la reconnaissance couvre tout ce que couvre whisper.

## Comment ça marche

```
Audio système  →  rééchantillonnage 16 kHz mono  →  détection de voix (Silero)
               →  reconnaissance vocale (whisper.cpp)
               →  traduction (llama.cpp, dans un processus séparé)
               →  transcription : original à gauche, traduction à côté
```

Tout ce qui se trouve sous l'interface est en Rust et tourne sur des threads
dédiés, et la traduction vit dans un binaire séparé car whisper.cpp et llama.cpp
ne peuvent pas partager un processus. La conception et ses raisons sont dans
[docs/ARCHITECTURE.md](../ARCHITECTURE.md).

Mesuré sur Apple Silicon avec les modèles par défaut, sur le corpus synthétique
de [tests/](../../tests/README.md) - médianes de trois passages par clip : le
premier sous-titre environ 6 secondes après le début, un nouveau toutes les 2-3
secondes, et un taux d'erreur par mot allant de 4 % sur une lecture claire à 23
% sur un clip de noms propres et de chiffres. La reconnaissance n'est pas
déterministe et un passage isolé varie d'une vingtaine de points : ce sont donc
des médianes et non des résultats ; la façon dont ces chiffres sont produits est
documentée à côté du banc d'essai.

## Plateformes

| Plateforme | État |
|---|---|
| **macOS 14.4+** | Prise en charge - Core Audio process taps, Metal |
| **Windows** | En développement - WASAPI loopback |
| **Linux** | En développement - PipeWire |

L'application se compile et démarre déjà sous Windows et Linux, mais la capture
audio s'y déclare indisponible : une fenêtre sans rien à écouter. Tout ce qui se
situe au-dessus de la capture est déjà indépendant de la plateforme.

Un pilote audio virtuel comme BlackHole n'est nécessaire sur **aucune**
plateforme : la capture passe par les API natives du système.

## Prérequis

| | |
|---|---|
| macOS | 14.4 ou plus récent, Apple Silicon ou Intel |
| Mémoire | 8 Go pour la reconnaissance seule, 16 Go avec la traduction |
| Disque | 0,1-6,5 Go pour les modèles choisis |
| Pour compiler | [Rust](https://rustup.rs), [Node.js](https://nodejs.org) 20+, cmake (`brew install cmake`) |

## Installation

### Le télécharger

La [dernière version](https://github.com/glenau/marswind/releases/latest)
contient un `.dmg`. Ouvrez-le, glissez Marswind dans Applications, c'est tout -
environ 13 Mo, les modèles étant téléchargés ensuite et seulement ceux que vous
choisissez.

**macOS refusera de l'ouvrir la première fois.** L'image est signée mais non
notarisée : aucun certificat Developer ID payant ne se cache derrière ce projet,
et Gatekeeper considère comme inconnu tout ce qui n'en a pas. Le passage :

1. Ouvrez l'app et laissez-la être bloquée. Appuyez sur **Done**, pas sur
   "Move to Bin".
2. **Réglages Système → Confidentialité et sécurité**, descendez jusqu'à
   **Sécurité**. Une ligne indique que Marswind a été bloqué, avec un bouton
   **Ouvrir quand même** à côté.
3. Appuyez dessus, authentifiez-vous par Touch ID ou mot de passe, puis
   confirmez encore une fois.

macOS ne demande qu'une fois. Le bouton n'apparaît qu'après un lancement bloqué
et dure environ une heure ; s'il n'y est pas, rouvrez l'app.

Le clic droit sur l'app puis Ouvrir était l'ancien raccourci pour cela et
fonctionne encore sur macOS 14. macOS 15 l'a supprimé, donc le chemin par les
réglages est celui qui marche partout.

### Ou le compiler

```bash
git clone https://github.com/glenau/marswind.git
cd marswind
npm install
npm run install:macos
```

Cela compile le worker de traduction, compile le bundle de release, le signe en
ad-hoc et le copie dans `/Applications/Marswind.app`. La première compilation
prend plusieurs minutes - whisper.cpp et llama.cpp sont compilés depuis les
sources. Rien d'autre n'est nécessaire : pas de sous-modules à
récupérer, pas de bibliothèques à installer à la main, pas de modèles à
télécharger au préalable.

### Premier lancement

1. `open /Applications/Marswind.app`
2. macOS demande l'autorisation **Enregistrement audio**. Acceptez - sans elle
   l'application n'entend rien. En cas de refus, elle se réaccorde dans Réglages
   Système → Confidentialité et sécurité → Enregistrement audio.
3. Ouvrez **Réglages** et téléchargez un modèle de reconnaissance et un modèle de
   traduction. `Large v3 Turbo (compressed)` et `Qwen3 4B Instruct` sont les
   valeurs par défaut à partir de 16 Go ; `Small` et `Qwen3 1.7B` tiennent dans
   8 Go. Environ 3 Go de téléchargement, vérifiés contre une somme de contrôle
   publiée. Chaque ligne indique la licence de ses poids - voir
   [docs/MODELS.md](../MODELS.md).
4. Appuyez sur **Commencer à écouter**, puis lancez quelque chose avec de la
   parole. Quatre extraits d'exemple sont dans les réglages, si vous préférez ne
   pas aller chercher une vidéo.

Deux choses à savoir sur une copie compilée soi-même :

- **Elle est signée en ad-hoc.** La signature est stable pour une compilation
  donnée, donc l'autorisation audio persiste - mais une recompilation crée une
  nouvelle identité et macOS redemande. C'est un certificat Developer ID qui met
  fin à cela, et il n'y en a pas encore.
- **Ne déplacez pas l'application pendant son exécution.** Pour la mettre à jour,
  relancez `npm run install:macos` ; il remplace `/Applications/Marswind.app` sur
  place.

### Mettre à jour

**Réglages → À propos → Rechercher des mises à jour.** L'application demande à
GitHub s'il existe une version plus récente ; si oui, elle télécharge l'image
dans Téléchargements, la compare à la somme de contrôle publiée à côté et
l'affiche dans le Finder. L'installer, c'est le même glisser que la première fois.

Rien ne se vérifie tout seul : pas de minuterie, pas de contrôle au lancement,
car l'application ne fait aucune requête réseau que vous n'avez pas déclenchée.

Une copie que vous avez compilée se met à jour comme elle a été installée :
`npm run install:macos` à nouveau.

### Créer une image disque

```bash
npm run build:dmg
```

Compile le bundle de release, le signe et l'empaquette dans
`src-tauri/target/Marswind-<version>-<arch>.dmg` - la même image que celle
jointe à une publication, avec la même réserve concernant Gatekeeper que
ci-dessus. La liste de contrôle est dans [docs/RELEASING.md](../RELEASING.md).

## Développement

`tauri dev` produit un exécutable nu sans `Info.plist` ni signature, et les
process taps de Core Audio refusent de fonctionner ainsi. Utilisez plutôt ceci -
il compile un bundle de debug, le signe et le lance :

```bash
npm run dev:macos
```

| Commande | Effet |
|---|---|
| `npm run dev:macos` | compiler, signer et lancer un bundle de debug |
| `npm run install:macos` | compiler un bundle de release et l'installer |
| `npm run check` | types Svelte et TypeScript |
| `npm run build:dmg` | un `.dmg` signé à transmettre à quelqu'un |
| `npm run build:sidecar` | le worker de traduction seul |
| `npm run build:icons` | redessiner l'icône depuis `scripts/make-icon.py` |
| `npm run build:social` | redessiner la carte que GitHub affiche sur un lien |
| `npm run licenses` | régénérer `THIRD-PARTY-NOTICES.md` depuis les lockfiles |

Il n'y a pas de CI : whisper.cpp et llama.cpp sont compilés depuis les sources
et le banc d'essai joue de l'audio par la sortie système, donc chaque
vérification est une commande locale. [CONTRIBUTING.md](../../CONTRIBUTING.md)
les liste.

## Tests

Les tests unitaires couvrent la logique pure ; les scripts de
[tests/](../../tests/README.md) jouent de l'audio par la sortie système et
notent ce qui ressort du vrai pipeline - reconnaissance, traduction et latence
ensemble.

```bash
npm run build:sidecar
cargo test --manifest-path src-tauri/Cargo.toml
```

La première ligne ne sert qu'une fois, puis seulement après un `cargo clean`.
Tauri embarque le worker de traduction comme sidecar, donc son script de build
refuse de construire `src-tauri` tant que le binaire n'existe pas : sur un clone
neuf, `cargo test` seul s'arrête sur
`resource path 'binaries/marswind-translator-…' doesn't exist`. Toute commande
`npm run` fait cette étape à votre place ; un appel direct à `cargo`, non.

Les scripts du banc d'essai ont besoin d'un bundle construit et signé, et de
modèles installés :

```bash
npm run dev:macos
tests/run-capture.sh
tests/run-asr.sh
tests/run-pipeline.sh
```

Une seule exécution sur le corpus varie d'une vingtaine de points de taux
d'erreur, donc un chiffre isolé ne veut rien dire. Comparez des médianes sur
plusieurs exécutions et lisez les transcriptions, pas seulement les scores.

## Confidentialité

- L'audio est capturé, rééchantillonné et reconnu **en mémoire**. Il n'est jamais
  écrit sur disque ni envoyé où que ce soit.
- Le seul trafic réseau est celui que vous déclenchez : télécharger un modèle ou
  rechercher une mise à jour. Rien ne part sur minuterie ni au lancement.
- Pas de télémétrie, pas d'analytique, pas de rapports de plantage, pas de compte.
- Les transcriptions sont écrites uniquement dans le dossier de données de
  l'application, pour que l'historique ait quelque chose à montrer. Elles se
  suppriment depuis l'application.

## Contribuer

Rapports de bugs, idées et pull requests sont les bienvenus.
[CONTRIBUTING.md](../../CONTRIBUTING.md) couvre l'installation, les
vérifications, la convention de commits et ce que la relecture regarde. Ouvrez
une issue avant de commencer quelque chose de conséquent - plusieurs
améliorations évidentes ont déjà été essayées puis annulées, mesures à l'appui.

- [Code de conduite](../../CODE_OF_CONDUCT.md)
- [Politique de sécurité](../../SECURITY.md) - signalez les vulnérabilités en
  privé, pas dans une issue

## Construit sur

| | | |
|---|---|---|
| [whisper.cpp](https://github.com/ggml-org/whisper.cpp) | MIT | la reconnaissance, et l'implémentation de Silero VAD avec elle |
| [llama.cpp](https://github.com/ggml-org/llama.cpp) | MIT | la traduction, dans son propre processus |
| [ggml](https://github.com/ggml-org/ggml) | MIT | la bibliothèque de tenseurs et le backend Metal sous les deux |
| [whisper-rs](https://codeberg.org/tazz4843/whisper-rs) | Unlicense | la liaison Rust vers whisper.cpp |
| [llama-cpp-2](https://github.com/utilityai/llama-cpp-rs) | MIT / Apache-2.0 | la liaison Rust vers llama.cpp |
| [Silero VAD](https://github.com/snakers4/silero-vad) | MIT | le modèle qui trouve les frontières de phrase |
| [Tauri](https://tauri.app) | MIT / Apache-2.0 | la fenêtre et la frontière entre processus |
| [Svelte](https://svelte.dev) | MIT | l'interface |
| [rubato](https://github.com/HEnquist/rubato) | MIT | le rééchantillonneur FFT devant whisper |

L'arbre de dépendances complet, avec la licence de chaque paquet, est dans
[THIRD-PARTY-NOTICES.md](../../THIRD-PARTY-NOTICES.md) - généré depuis les
lockfiles et embarqué dans l'application à côté de la licence elle-même.

**Rien de tout cela ne couvre les modèles.** Ils sont téléchargés depuis
[Hugging Face](https://huggingface.co) à votre demande et conservent les
conditions de ceux qui les publient - et seuls ceux dont les conditions
s'acceptent sans les lire entrent au catalogue : les modèles whisper et Silero
sont sous MIT, Qwen3 sous Apache-2.0. Chaque ligne des réglages nomme sa licence
avant que le téléchargement ne commence. Le détail est dans
[docs/MODELS.md](../MODELS.md).

## Licence

MIT - voir [LICENSE](../../LICENSE). Cela couvre ce dépôt ; cela ne couvre pas
les modèles, et les mentions ci-dessus ne remplacent pas les licences vers
lesquelles elles pointent.