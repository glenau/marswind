import type { Dictionary } from "./en";

const ru: Dictionary = {
  strings: {
    "stage.audio": "Звук",
    "stage.recognition": "Распознавание",
    "stage.translation": "Перевод",
    "stage.notCapturing": "Не записывается",
    "stage.notRunning": "Не запущено",
    "meter.level": "Уровень звука",
    "action.start": "Начать",
    "action.stop": "Остановить",
    "action.working": "Запуск…",
    "action.history": "История",
    "action.settings": "Настройки",
    "action.clear": "Очистить",
    "action.dismiss": "Скрыть",
    "action.refresh": "Обновить",
    "action.delete": "Удалить",
    "action.play": "Слушать",
    "action.text": "Текст",
    "action.hideText": "Скрыть текст",
    "action.install": "Установить",
    "action.remove": "Удалить",
    "action.cancel": "Отменить",
    "action.github": "Проект на GitHub",

    "transcript.original": "Оригинал",
    "transcript.translation": "Перевод",
    "transcript.into": "Перевод на",
    "transcript.toLanguage": "на",
    "transcript.emptySource":
      "Нажмите «Начать» и включите что-нибудь с речью - или проиграйте образец в настройках. Здесь появится то, что было сказано.",
    "transcript.emptyTarget":
      "Здесь появится перевод - по частям, пока говорящий ещё продолжает.",
    "transcript.translating": "перевод…",
    "transcript.notTranslated": "- не переведено",
    "transcript.toBottom": "Следить за последней строкой",

    "settings.general": "Основное",
    "settings.models": "Модели",
    "settings.about": "О программе",
    "settings.audioTitle": "Звук",
    "settings.audioNote":
      "Откуда берётся звук. Либо всё, что играет на этом компьютере, либо одно приложение.",
    "settings.audio": "Источник звука",
    "settings.recognition": "Распознавание речи",
    "settings.recognitionNote":
      "Превращает то, что играет на компьютере, в текст. Нужна модель распознавания.",
    "settings.translation": "Перевод",
    "settings.translationNote":
      "Превращает распознанный текст в другой язык. Нужна модель перевода - отдельная от той, что выше.",
    "settings.on": "Вкл",
    "settings.showOriginal": "Показывать оригинал рядом с переводом",
    "settings.locked": "Остановите запись, чтобы менять эти настройки.",
    "settings.noModel": "Модель не установлена",
    "settings.needVad":
      "Установите модель определения речи ниже - без неё распознавание не работает.",
    "settings.needAsr": "Установите модель распознавания ниже.",
    "settings.needMt": "Установите модель перевода ниже.",
    "settings.detect": "Определять автоматически",
    "settings.model": "Модель",
    "settings.interface": "Интерфейс",
    "settings.interfaceNote":
      "Как выглядит само окно. На то, что приложение делает со звуком, это не влияет.",
    "settings.language": "Язык",
    "settings.theme": "Тема",
    "settings.textSize": "Размер текста",
    "settings.samples": "Попробовать без поиска видео",
    "settings.samplesNote":
      "Проигрывает записанный отрывок через динамики, поэтому он идёт тем же путём, что и любой другой звук. Сначала нажмите «Начать».",

    "theme.dark": "Тёмная",
    "theme.light": "Светлая",
    "theme.system": "Как в системе",

    "about.title": "О программе",
    "about.tagline":
      "Живые субтитры и перевод для всего, что играет на этом компьютере. Распознавание и перевод работают здесь же - без аккаунта, без ключей и без единого байта наружу.",
    "about.version": "Версия",
    "about.runtime": "Среда",
    "about.license": "Лицензия",
    "about.source": "Исходный код",
    "about.issues": "Сообщить о проблеме",
    "about.licenseFile": "Текст лицензии",
    "about.notices": "Лицензии компонентов",
    "about.built": "Собрано на",
    "about.how": "Как это работает",
    "about.howCapture":
      "Звук берётся у самой macOS, через перехват процесса в Core Audio: ровно то, что этот компьютер уже играет - всё сразу или одно приложение. Ни виртуального аудиодрайвера, ни микрофона, и то, что вы слышите, не меняется.",
    "about.howPipeline":
      "Этот поток сводится в моно на 16 кГц и остаётся в памяти. Silero VAD режет его на фразы, whisper.cpp превращает их в текст на видеокарте, а llama.cpp переводит в отдельном процессе, пока говорящий ещё продолжает. Сам звук никуда не записывается и не покидает этот компьютер.",

    "models.title": "Модели",
    "models.note":
      "Всё работает на этом компьютере, поэтому каждая модель - файл на этом диске. Нужна одна для распознавания и одна для перевода.",
    "models.onDisk": "на диске",
    "models.forRecognition": "распознавание",
    "models.forTranslation": "перевод",
    "models.forVad": "определение речи",
    "models.recommended": "рекомендуется",
    "models.required": "обязательна",
    "models.installed": "установлена",
    "models.of": "из",

    "history.empty":
      "Пока пусто. Каждый раз, когда вы нажимаете «Начать», распознанное и переведённое пишется сюда.",
    "history.pick": "Выберите сессию слева.",
    "history.loading": "Загрузка…",
    "history.rows": "строк",
    "history.words": "слов",
    "history.exportText": "Выгрузить текст",
    "history.exportSrt": "Выгрузить субтитры",
    "history.exportJson": "Выгрузить JSON",
    "history.saved": "Сохранено в",
    "history.noModel": "без модели распознавания",
    "history.notTranslated": "не переведено",
    "history.segment": "сегмент",
    "history.segments": "сегментов",

    "size.small": "Мелкий",
    "size.medium": "Обычный",
    "size.large": "Крупный",
    "size.huge": "Очень крупный",
  },
  phrases: {
    idle: [
      "Привет! Переведём что-нибудь?",
      "Включите что угодно, где говорят",
      "Готов, когда скажете",
      "Тихо. Но я всё равно слушаю",
      "Одно нажатие - и пойдут слова",
      "Ничего не уходит с этого компьютера",
      "Что сегодня смотрим?",
      "Подкаст, лекция, созвон - мне всё равно",
      "Жду первую фразу",
      "Нажмите «Начать» - и поехали",
    ],
    live: [
      "Внимательно слушаю…",
      "Пытаюсь разобрать слова…",
      "Ловлю фразы на лету…",
      "Так, это точно было слово…",
      "Листаю словарь в голове…",
      "Успеваю, почти…",
      "Перевожу быстрее, чем думаю…",
      "Держу ухо востро…",
      "Ещё чуть-чуть - и будет фраза…",
      "Слушаю и не отвлекаюсь…",
    ],
  },
};

export default ru;
