#!/usr/bin/env python3
"""Generate translations/ru/LC_MESSAGES/nothingx.po from the @tr() strings in ui/main.slint.
Fails if a string has no Russian translation."""
import re, sys, pathlib

RU = {
    'CONTROLS': 'УПРАВЛЕНИЕ',
    'DEVICE': 'УСТРОЙСТВО',
    'SETTINGS': 'НАСТРОЙКИ',
    'NOISE CONTROL': 'УПРАВЛЕНИЕ ШУМОМ',
    'SPATIAL AUDIO': 'ПРОСТРАНСТВЕННЫЙ ЗВУК',
    'BASS': 'БАСЫ',
    'LATENCY': 'ЗАДЕРЖКА',
    'Controls': 'Управление',
    'Gestures and buttons': 'Жесты и кнопки',
    'Device settings': 'Настройки устройства',
    'Simple': 'Простой',
    'Advanced': 'Расширенный',
    'PRESETS': 'ПРЕСЕТЫ',
    'CUSTOM': 'СВОЙ ЗВУК',
    'Move a slider to switch to your own sound. Changes apply when you release it.': 'Сдвиньте ползунок, чтобы включить свой звук. Изменения применяются, когда вы его отпускаете.',
    'ADVANCED EQ': 'РАСШИРЕННЫЙ ЭКВАЛАЙЗЕР',
    'Drag the points or pick a band below. Up to 8 bands.': 'Перетаскивайте точки или выберите полосу ниже. До 8 полос.',
    'Add band': 'Добавить полосу',
    'Band {}': 'Полоса {}',
    'Low shelf': 'НЧ-полка',
    'Peak': 'Колокол',
    'High shelf': 'ВЧ-полка',
    'Remove band': 'Удалить полосу',
    'Frequency': 'Частота',
    'Gain': 'Усиление',
    'Width (Q)': 'Ширина (Q)',
    'Connect your headphones to change the controls.': 'Подключите наушники, чтобы настроить управление.',
    'This device has no customizable controls.': 'У этого устройства нет настраиваемого управления.',
    'Device': 'Устройство',
    'Connect your headphones to see their settings.': 'Подключите наушники, чтобы увидеть их настройки.',
    'FEATURES': 'ФУНКЦИИ',
    'In-ear detection': 'Датчик ношения',
    'Pause playback when you take the headphones off.': 'Музыка встаёт на паузу, когда вы снимаете наушники.',
    'Personalised noise cancellation': 'Персональное шумоподавление',
    'Tune noise cancellation to the shape of your ears.': 'Шумоподавление подстраивается под форму ваших ушей.',
    'CONNECTIONS': 'ПОДКЛЮЧЕНИЯ',
    'Dual connection': 'Двойное подключение',
    'Stay connected to two devices at once.': 'Наушники остаются подключёнными сразу к двум устройствам.',
    'Connected': 'Подключено',
    'TOOLS': 'ИНСТРУМЕНТЫ',
    'Find my device': 'Найти устройство',
    'Plays a loud sound. Take the headphones off first.': 'Наушники издадут громкий звук — сначала снимите их.',
    'Stop': 'Остановить',
    'Play sound': 'Воспроизвести звук',
    'Stop left': 'Остановить левый',
    'Left': 'Левый',
    'Stop right': 'Остановить правый',
    'Right': 'Правый',
    'Ear tip fit test': 'Тест посадки амбушюр',
    'Put both earbuds in and start the test. It plays a short sound and checks the seal.': 'Вставьте оба наушника и начните тест: прозвучит короткий сигнал и будет проверена изоляция.',
    'Testing…': 'Проверка…',
    'Start test': 'Начать тест',
    'Left: {}': 'Левый: {}',
    'Right: {}': 'Правый: {}',
    'ABOUT': 'О ПРИЛОЖЕНИИ',
    'ABOUT DEVICE': 'ОБ УСТРОЙСТВЕ',
    'Settings': 'Настройки',
    'APP': 'ПРИЛОЖЕНИЕ',
    'Show the charge next to the menu bar icon.': 'Показывать заряд рядом с иконкой в строке меню.',
    'Open at login': 'Открывать при входе',
    'Start NothingX in the menu bar when you log in.': 'Запускать NothingX в строке меню при входе в систему.',
    'Unofficial app, not affiliated with Nothing Technology Limited. Nothing and CMF are trademarks of their owner.': 'Неофициальное приложение, не связано с Nothing Technology Limited. Nothing и CMF — товарные знаки их владельца.',
    'Made with Slint. Fonts: Inter, Matrix Sans, Instrument Serif (SIL Open Font License).': 'Сделано на Slint. Шрифты: Inter, Matrix Sans, Instrument Serif (SIL Open Font License).',
    'Open NothingX': 'Открыть NothingX',
    'Connecting…': 'Подключение…',
    'Quit': 'Выйти',
    'Quit NothingX': 'Выйти из NothingX',
    "DEVICES": "УСТРОЙСТВА",
    "EQUALIZER": "ЭКВАЛАЙЗЕР",
    "ADD": "ДОБАВИТЬ",
    "ACCOUNT": "АККАУНТ",
    "Devices": "Устройства",
    "My page": "Моя страница",
    "Add device": "Добавить устройство",
    "Bluetooth access is off": "Нет доступа к Bluetooth",
    "Allow NothingX in System Settings → Privacy & Security → Bluetooth, then reopen the app.":
        "Разрешите NothingX в Системных настройках → Конфиденциальность и безопасность → Bluetooth и перезапустите приложение.",
    "No Nothing or CMF device found": "Устройства Nothing или CMF не найдены",
    "Pair your headphones in System Settings → Bluetooth. They show up here automatically.":
        "Подключите наушники в Системных настройках → Bluetooth — они появятся здесь автоматически.",
    "Open Bluetooth settings": "Открыть настройки Bluetooth",
    "CHARGING": "ЗАРЯДКА",
    "Connecting to the headphones…": "Подключаемся к наушникам…",
    "Headphones are not connected": "Наушники не подключены",
    "Turn the headphones on and connect them to this Mac. If they are connected but controls don't appear, close Nothing X on your phone — the headphones accept only one control app at a time.":
        "Включите наушники и подключите их к этому Mac. Если они подключены, а управление не появляется, закройте Nothing X на телефоне: наушники работают только с одним приложением управления одновременно.",
    "Noise control": "Шумоподавление",
    "Noise cancellation · adaptive": "Шумоподавление · адаптивное",
    "Noise cancellation · low": "Шумоподавление · низкое",
    "Noise cancellation · mid": "Шумоподавление · среднее",
    "Noise cancellation · high": "Шумоподавление · высокое",
    "Transparency": "Прозрачность",
    "Off": "Выкл.",
    "On": "Вкл.",
    "Noise cancellation": "Шумоподавление",
    "High": "Высокое",
    "Mid": "Среднее",
    "Low": "Низкое",
    "Adaptive": "Адаптивное",
    "Spatial audio": "Пространственный звук",
    "Head tracking": "Отслеживание головы",
    "Fixed": "Фиксированный",
    "Bass enhance": "Усиление басов",
    "On · level {}": "Вкл. · уровень {}",
    "Level {}": "Уровень {}",
    "Low latency mode": "Режим низкой задержки",
    "Equalizer": "Эквалайзер",
    "About device": "Об устройстве",
    "Firmware {}": "Прошивка {}",
    "S/N {}": "S/N {}",
    " · tested": " · проверено",
    " · best effort": " · экспериментально",
    "Connect your headphones to adjust the equalizer.": "Подключите наушники, чтобы настроить эквалайзер.",
    "Presets": "Пресеты",
    "Custom": "Свой",
    "Custom equalizer": "Свой эквалайзер",
    "Active. Changes apply when you release a slider.": "Активен. Изменения применяются, когда вы отпускаете ползунок.",
    "Move a slider to switch to the custom profile.": "Сдвиньте ползунок, чтобы включить свой профиль.",
    "Reset": "Сбросить",
    "Bass": "Низкие",
    "Mids": "Средние",
    "Treble": "Высокие",
    "Connection guide": "Руководство по подключению",
    "How to add headphones": "Как добавить наушники",
    "1. Put the headphones in pairing mode (pick your model below for the exact steps).\n2. Pair them in System Settings → Bluetooth.\n3. NothingX finds them automatically.":
        "1. Переведите наушники в режим сопряжения (точные шаги — в карточке вашей модели ниже).\n2. Подключите их в Системных настройках → Bluetooth.\n3. NothingX найдёт их автоматически.",
    "1. Turn the headphones on.\n2. Press and hold the power / Bluetooth button until the LED starts flashing.\n3. Choose the headphones in System Settings → Bluetooth.":
        "1. Включите наушники.\n2. Нажмите и удерживайте кнопку питания / Bluetooth, пока индикатор не начнёт мигать.\n3. Выберите наушники в Системных настройках → Bluetooth.",
    "1. Turn the neckband on.\n2. Press and hold the power button until the LED starts flashing.\n3. Choose it in System Settings → Bluetooth.":
        "1. Включите наушники.\n2. Удерживайте кнопку питания, пока индикатор не начнёт мигать.\n3. Выберите их в Системных настройках → Bluetooth.",
    "1. Put both earbuds in the case and keep it open.\n2. Press and hold the button on the case until the LED flashes.\n3. Choose them in System Settings → Bluetooth.":
        "1. Положите оба наушника в кейс и не закрывайте его.\n2. Удерживайте кнопку на кейсе, пока индикатор не замигает.\n3. Выберите их в Системных настройках → Bluetooth.",
    "1. Put both earbuds in the case and open the lid.\n2. Press and hold the button on the case for about 2 seconds until the LED flashes.\n3. Choose them in System Settings → Bluetooth.":
        "1. Положите оба наушника в кейс и откройте крышку.\n2. Удерживайте кнопку на кейсе около 2 секунд, пока индикатор не замигает.\n3. Выберите их в Системных настройках → Bluetooth.",
    "Tested with NothingX.": "Проверено с NothingX.",
    "Supported on a best-effort basis: controls use the shared Nothing protocol.":
        "Поддержка экспериментальная: управление идёт по общему протоколу Nothing.",
    "Close": "Закрыть",
    "Firmware updates are installed with the Nothing X app on your phone.":
        "Прошивка обновляется через приложение Nothing X на телефоне.",
    "1. Turn the headphones on.\n2. Press and hold the Bluetooth button for 2–3 seconds, until the light flashes white.\n3. Choose the headphones in System Settings → Bluetooth.":
        "1. Включите наушники.\n2. Удерживайте кнопку Bluetooth 2–3 секунды, пока индикатор не замигает белым.\n3. Выберите наушники в Системных настройках → Bluetooth.",
    "1. Put both earbuds in your ears.\n2. Touch and hold the touch areas of both earbuds at the same time for 5 seconds.\n3. Choose them in System Settings → Bluetooth.":
        "1. Вставьте оба наушника в уши.\n2. Одновременно коснитесь сенсорных зон обоих наушников и удерживайте 5 секунд.\n3. Выберите их в Системных настройках → Bluetooth.",
    "1. Turn the neckband on.\n2. Press and hold the power button until the light starts flashing.\n3. Choose it in System Settings → Bluetooth.":
        "1. Включите наушники.\n2. Удерживайте кнопку питания, пока индикатор не начнёт мигать.\n3. Выберите их в Системных настройках → Bluetooth.",
    "1. Put both earbuds in the case and leave the lid open.\n2. Press and hold the button on the case for about 2 seconds, until the light flashes white.\n3. Choose them in System Settings → Bluetooth.":
        "1. Положите оба наушника в кейс и не закрывайте крышку.\n2. Удерживайте кнопку на кейсе около 2 секунд, пока индикатор не замигает белым.\n3. Выберите их в Системных настройках → Bluetooth.",
    "Once paired, NothingX connects to them automatically. If the controls don't appear, close Nothing X on your phone.":
        "После сопряжения NothingX подключится к ним сам. Если управление не появилось, закройте Nothing X на телефоне.",
    "Noise control off": "Шумоподавление выключено",
    "Nothing Account": "Аккаунт Nothing",
    "Your name": "Ваше имя",
    "Email": "Эл. почта",
    "Preferences": "Предпочтения",
    "Battery in the menu bar": "Заряд в строке меню",
    "Language": "Язык",
    "System": "Как в системе",
    "About": "О приложении",
    "NothingX for macOS {}": "NothingX для macOS {}",
    "Unofficial app, not affiliated with Nothing Technology. Your profile stays on this Mac: Nothing cloud sign-in isn't available to third-party apps.":
        "Неофициальное приложение, не связано с Nothing Technology. Профиль хранится только на этом Mac: вход в облако Nothing сторонним приложениям недоступен.",
    "Open log": "Открыть журнал",
    "Choose photo": "Выбрать фото",
    "Photo": "Фото",
    "Remove": "Удалить",
    "Remove photo": "Удалить фото",
}

root = pathlib.Path(__file__).resolve().parent.parent
src = (root / "ui/main.slint").read_text()
raw = re.findall(r'@tr\("((?:[^"\\]|\\.)*)"', src)
ids = []
for r in raw:
    s = r.encode().decode("unicode_escape").encode("latin-1").decode("utf-8")
    if s not in ids:
        ids.append(s)
missing = [s for s in ids if s not in RU]
if missing:
    sys.exit("missing translations:\n" + "\n".join(repr(m) for m in missing))

def q(s):
    return '"' + s.replace("\\", "\\\\").replace('"', '\\"').replace("\n", "\\n") + '"'

out = ['msgid ""', 'msgstr ""', '"Content-Type: text/plain; charset=UTF-8\\n"', '"Language: ru\\n"', ""]
for s in ids:
    out += [f"msgid {q(s)}", f"msgstr {q(RU[s])}", ""]
(root / "translations/ru/LC_MESSAGES/nothingx.po").write_text("\n".join(out))
print(f"{len(ids)} strings translated")
