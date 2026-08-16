import type { Dictionary } from "./en";

const uk: Dictionary = {
  strings: {
    "stage.audio": "Звук",
    "stage.recognition": "Розпізнавання",
    "stage.translation": "Переклад",
    "stage.notCapturing": "Не записується",
    "stage.notRunning": "Не запущено",
    "meter.level": "Рівень звуку",
    "action.start": "Почати",
    "action.stop": "Зупинити",
    "action.working": "Запуск…",
    "action.history": "Історія",
    "action.settings": "Налаштування",
    "action.clear": "Очистити",
    "action.dismiss": "Сховати",
    "action.refresh": "Оновити",
    "action.delete": "Видалити",
    "action.play": "Відтворити",
    "action.text": "Текст",
    "action.hideText": "Сховати текст",
    "action.install": "Встановити",
    "action.remove": "Вилучити",
    "action.cancel": "Скасувати",
    "action.github": "Проєкт на GitHub",

    "transcript.original": "Оригінал",
    "transcript.translation": "Переклад",
    "transcript.into": "Переклад на",
    "transcript.toLanguage": "на",
    "transcript.emptySource":
      "Натисніть «Почати» і увімкніть щось із мовленням - або програйте зразок у налаштуваннях. Тут з’явиться те, що було сказано.",
    "transcript.emptyTarget":
      "Тут з’явиться переклад - частинами, поки мовець ще говорить.",
    "transcript.translating": "переклад…",
    "transcript.notTranslated": "- не перекладено",
    "transcript.toBottom": "Стежити за останнім рядком",

    "settings.general": "Основне",
    "settings.models": "Моделі",
    "settings.about": "Про програму",
    "settings.audioTitle": "Звук",
    "settings.audioNote":
      "Звідки береться звук. Або все, що грає на цьому комп’ютері, або один застосунок.",
    "settings.audio": "Джерело звуку",
    "settings.recognition": "Розпізнавання мовлення",
    "settings.recognitionNote":
      "Перетворює те, що грає на комп’ютері, на текст. Потрібна модель розпізнавання.",
    "settings.translation": "Переклад",
    "settings.translationNote":
      "Перекладає розпізнаний текст іншою мовою. Потрібна модель перекладу - окрема від тієї, що вище.",
    "settings.on": "Увімк.",
    "settings.showOriginal": "Показувати оригінал поруч із перекладом",
    "settings.locked": "Зупиніть запис, щоб змінити ці налаштування.",
    "settings.noModel": "Модель не встановлено",
    "settings.needVad":
      "Встановіть модель визначення мовлення нижче - без неї розпізнавання не працює.",
    "settings.needAsr": "Встановіть модель розпізнавання нижче.",
    "settings.needMt": "Встановіть модель перекладу нижче.",
    "settings.detect": "Визначати автоматично",
    "settings.model": "Модель",
    "settings.interface": "Інтерфейс",
    "settings.interfaceNote":
      "Як виглядає саме вікно. На те, що застосунок робить зі звуком, це не впливає.",
    "settings.language": "Мова",
    "settings.theme": "Тема",
    "settings.textSize": "Розмір тексту",
    "settings.samples": "Спробувати без пошуку відео",
    "settings.samplesNote":
      "Програє записаний уривок через динаміки, тож він іде тим самим шляхом, що й будь-який інший звук. Спершу натисніть «Почати».",

    "theme.dark": "Темна",
    "theme.light": "Світла",
    "theme.system": "Як у системі",

    "about.title": "Про програму",
    "about.tagline":
      "Живі субтитри та переклад усього, що грає на цьому комп’ютері. Розпізнавання й переклад працюють тут-таки - без облікового запису, без ключів і без жодного байта назовні.",
    "about.version": "Версія",
    "about.runtime": "Середовище",
    "about.license": "Ліцензія",
    "about.source": "Вихідний код",
    "about.issues": "Повідомити про проблему",
    "about.licenseFile": "Текст ліцензії",
    "about.notices": "Ліцензії компонентів",
    "about.built": "Зроблено на",
    "about.how": "Як це працює",
    "about.howCapture":
      "Звук береться в самої macOS, через перехоплення процесу в Core Audio: саме те, що цей комп’ютер уже відтворює - усе відразу або один застосунок. Без віртуального аудіодрайвера, без мікрофона, і те, що ви чуєте, не змінюється.",
    "about.howPipeline":
      "Цей потік зводиться в моно на 16 кГц і лишається в пам’яті. Silero VAD ріже його на фрази, whisper.cpp перетворює їх на текст на відеокарті, а llama.cpp перекладає в окремому процесі, поки мовець ще говорить. Сам звук ніколи не записується на диск і не виходить за межі цього комп’ютера.",

    "models.title": "Моделі",
    "models.note":
      "Усе працює на цьому комп’ютері, тож кожна модель - файл на цьому диску. Потрібна одна для розпізнавання і одна для перекладу.",
    "models.onDisk": "на диску",
    "models.forRecognition": "розпізнавання",
    "models.forTranslation": "переклад",
    "models.forVad": "визначення мовлення",
    "models.recommended": "рекомендовано",
    "models.required": "обов’язкова",
    "models.installed": "встановлено",
    "models.of": "з",

    "history.empty":
      "Поки порожньо. Щоразу, коли ви натискаєте «Почати», розпізнане й перекладене записується сюди.",
    "history.pick": "Виберіть сесію ліворуч.",
    "history.loading": "Завантаження…",
    "history.rows": "рядків",
    "history.words": "слів",
    "history.exportText": "Вивантажити текст",
    "history.exportSrt": "Вивантажити субтитри",
    "history.exportJson": "Вивантажити JSON",
    "history.saved": "Збережено в",
    "history.noModel": "без моделі розпізнавання",
    "history.notTranslated": "не перекладено",
    "history.segment": "сегмент",
    "history.segments": "сегментів",

    "size.small": "Дрібний",
    "size.medium": "Звичайний",
    "size.large": "Великий",
    "size.huge": "Дуже великий",
  },
  phrases: {
    idle: [
      "Привіт! Перекладемо щось?",
      "Увімкніть будь-що, де говорять",
      "Готовий, щойно скажете",
      "Тихо. Але я все одно слухаю",
      "Одне натискання - і підуть слова",
      "Нічого не виходить за межі цього комп’ютера",
      "Що сьогодні дивимось?",
      "Подкаст, лекція, дзвінок - мені байдуже",
      "Чекаю на першу фразу",
      "Натисніть «Почати» - і поїхали",
    ],
    live: [
      "Уважно слухаю…",
      "Намагаюся розібрати слова…",
      "Ловлю фрази на льоту…",
      "Так, це точно було слово…",
      "Гортаю словник у голові…",
      "Встигаю, майже…",
      "Перекладаю швидше, ніж думаю…",
      "Нашорошив вуха…",
      "Ще трохи - і буде фраза…",
      "Слухаю й не відволікаюся…",
    ],
  },
};

export default uk;
