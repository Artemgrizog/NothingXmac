# NothingX for Mac

[![Made with Slint](https://img.shields.io/badge/Made%20with-Slint-2379F4)](https://slint.dev)

**English** · [Русский](#русский)

An unofficial, native companion app for **Nothing** and **CMF** headphones on Apple Silicon Macs —
the Nothing X experience on macOS. Written in Rust with the [Slint](https://slint.dev) UI toolkit, talking
to the headphones directly over Bluetooth (IOBluetooth RFCOMM). No Electron, no web view, no network access.

> Not affiliated with Nothing Technology Limited. "Nothing" and "CMF" are trademarks of their owner.

## Features

- **Menu bar panel** — noise control (with levels), battery for each earbud / case, quick links; battery % next to the icon.
- **Noise control** — noise cancellation (high / mid / low / adaptive), transparency, off.
- **Equalizer** — presets (incl. CMF / Dirac Opteo presets), custom 3-band EQ and an **8-band parametric "Advanced" EQ** with a live response graph.
- **Controls** — customize gestures and buttons (pinch / tap / press, Headphone (1) button and roller, CMF case knob…).
- **Device settings** — in-ear detection, dual connection (with the list of paired hosts), personalised ANC, find my device, ear tip fit test, firmware / serial info.
- **Spatial audio** (head tracking / fixed), **bass enhance** with levels, **low latency mode**.
- Connection guide for every Nothing and CMF model, Russian / English UI, open at login.

## Supported devices

| Status | Models |
| --- | --- |
| Tested on hardware | Headphone (1) |
| Supported (shared Nothing protocol) | Headphone (1) Pro, Headphone (a), Ear (3), Ear (3a), Ear, Ear (2), Ear (1), Ear (a), Ear (stick), Ear (open), CMF Headphone Pro, CMF Buds, CMF Buds 2, CMF Buds 2a, CMF Buds 2 Plus, CMF Buds Pro, CMF Buds Pro 2, CMF Buds Neo, CMF Neckband Pro |

Reports for untested models are very welcome (Settings → Open log helps a lot).

## Install

1. Download `NothingX-<version>.dmg` from Releases, open it and drag **NothingX** to **Applications**.
2. The app is not notarized by Apple. On first launch macOS will block it: open
   **System Settings → Privacy & Security** and click **Open Anyway** (or right-click the app → Open).
   Alternatively run `xattr -dr com.apple.quarantine /Applications/NothingX.app`.
3. Allow Bluetooth access when asked.

The headphones accept only **one control app at a time** — if controls don't appear, close Nothing X on your phone.

Requires macOS 13 or later on Apple Silicon.

## Build from source

```bash
./scripts/package.sh       # release build → dist/NothingX.app and dist/NothingX-<version>.dmg
./scripts/dev.sh           # debug build in an app bundle (needed for the Bluetooth permission)
cargo test                 # protocol and catalog tests
python3 scripts/make_po.py # regenerate the Russian translation after changing @tr() strings
```

Needs Rust (stable) and Xcode command line tools; `package.sh` installs [dmgbuild](https://github.com/dmgbuild/dmgbuild) into `.venv-dmg`.

## How it works

The headphones expose a serial control channel ("NT LINK", RFCOMM, UUID `aeac4a03-dff5-498f-843a-34487cf133eb`).
Frames are `55 60 01 | command (LE) | length (LE) | sequence | payload | CRC-16/MODBUS (LE)`.
See `src/proto.rs` and `src/catalog.rs` for the commands and the per-model capabilities.

Protocol knowledge builds on community research, especially [BudsLink](https://github.com/maniacx/BudsLink)
and [this Ear (2) write-up](https://bharadwaj-raju.github.io/posts/nothing-ear-2-on-linux), plus testing on real hardware.

## Files

- Settings: `~/Library/Application Support/NothingX/settings.json`
- Log: `~/Library/Logs/NothingX.log`
- Custom device images: `~/Library/Application Support/NothingX/images/<model code>.png` overrides the built-in ones.

## License

Code: MIT (see `LICENSE`). Fonts: SIL OFL 1.1. Device images © Nothing Technology Limited.
UI built with [Slint](https://slint.dev) under its royalty-free license.

---

## Русский

Неофициальное приложение для наушников **Nothing** и **CMF** на Mac с Apple Silicon — Nothing X для macOS.
Написано на Rust ([Slint](https://slint.dev)), работает с наушниками напрямую по Bluetooth. Без Electron и браузера, без доступа в интернет.

> Не связано с Nothing Technology Limited. «Nothing» и «CMF» — товарные знаки их владельца.

### Возможности

- **Панель в строке меню** — шумоподавление с уровнями, заряд наушников и кейса, быстрые переходы; процент заряда рядом с иконкой.
- **Шумоподавление** — высокое / среднее / низкое / адаптивное, прозрачность, выкл.
- **Эквалайзер** — пресеты (в т.ч. Dirac Opteo у CMF), свой 3-полосный и **продвинутый 8-полосный** с графиком АЧХ.
- **Управление** — настройка жестов и кнопок.
- **Настройки устройства** — датчик ношения, двойное подключение со списком устройств, персональное шумоподавление, поиск наушников, тест посадки амбушюр, прошивка и серийный номер.
- **Пространственный звук**, **оптимизация баса**, **режим малой задержки**, руководство по подключению, автозапуск.

Проверено на **Headphone (1)**; остальные модели поддерживаются по общему протоколу Nothing (отмечены как экспериментальные).

### Установка

1. Скачайте `NothingX-<версия>.dmg`, откройте и перетащите **NothingX** в **Программы**.
2. Приложение не нотаризовано Apple: при первом запуске откройте **Системные настройки → Конфиденциальность и безопасность**
   и нажмите **«Всё равно открыть»** (или правый клик по приложению → «Открыть»).
3. Разрешите доступ к Bluetooth.

Наушники принимают только одно приложение управления — если управление не появляется, закройте Nothing X на телефоне.

Нужна macOS 13 или новее на Apple Silicon.

### Сборка

```bash
./scripts/package.sh   # dist/NothingX.app и dist/NothingX-<версия>.dmg
```
