# NothingX for Mac

[![Made with Slint](https://img.shields.io/badge/Made%20with-Slint-2379F4)](https://slint.dev)

**English** · [Русский](#русский)

An unofficial, native companion app for **Nothing** and **CMF** headphones on Apple Silicon Macs —
the Nothing X experience on macOS. Written in Rust with the [Slint](https://slint.dev) UI toolkit, talking
to the headphones directly over Bluetooth (IOBluetooth RFCOMM). No Electron, no web view, no network access.

> Not affiliated with Nothing Technology Limited. "Nothing" and "CMF" are trademarks of their owner.

## Features

- **Menu bar panel** — switch noise control and its level, see the battery of each earbud and the case, jump to the equalizer or controls; battery % next to the menu bar icon. Works over full-screen apps.
- **Noise control** — noise cancellation (high / mid / low / adaptive), transparency, off.
- **Equalizer** — presets (incl. CMF / Dirac Opteo presets), custom 3-band EQ and an **8-band parametric "Advanced" EQ** with a live response graph.
- **Controls** — customize gestures and buttons (pinch / tap / press, Headphone (1) button and roller, CMF case knob…).
- **Device settings** — in-ear detection, dual connection (with the list of paired hosts), personalised ANC, find my device, ear tip fit test, firmware version, serial number.
- **Spatial audio** (head tracking / fixed), **bass enhance** with levels, **low latency mode**.
- Pairing guide for every Nothing and CMF model, Russian / English UI, open at login.

**Firmware updates** are not part of the app: Nothing's update protocol and firmware files are not public,
and a failed update can brick the headphones. Update them with the Nothing X app on your phone; NothingX shows
the installed version.

## Supported devices

**Nothing:** Headphone (1), Headphone (1) Pro, Headphone (a), Ear (3), Ear (3a), Ear, Ear (2), Ear (1), Ear (a), Ear (stick), Ear (open).
**CMF:** Headphone Pro, Buds, Buds 2, Buds 2a, Buds 2 Plus, Buds Pro, Buds Pro 2, Buds Neo, Neckband Pro.

All models share the Nothing control protocol; development and testing happened on a Headphone (1).
Feature sets per model follow `src/catalog.rs`. If something doesn't work on your model, please open an
issue and attach the log (Settings → Open log).

## Install

1. Download `NothingX-<version>.dmg` from Releases, open it and drag **NothingX** to **Applications**.
2. The app is not notarized by Apple, so on first launch macOS blocks it ("Apple could not verify…").
   Click **Done**, open **System Settings → Privacy & Security**, scroll down and click **Open Anyway**,
   then confirm with your password. You only need to do this once.
   Alternatively run `xattr -dr com.apple.quarantine /Applications/NothingX.app` in Terminal.
3. Allow Bluetooth access when asked. NothingX lives in the menu bar; closing the window keeps it running there
   (use **Quit** in the panel to exit).

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

- **Панель в строке меню** — переключение шумоподавления и его уровня, заряд наушников и кейса, переход к эквалайзеру и управлению; процент заряда рядом с иконкой. Открывается и поверх полноэкранных приложений.
- **Шумоподавление** — высокое / среднее / низкое / адаптивное, прозрачность, выкл.
- **Эквалайзер** — пресеты (в т.ч. Dirac Opteo у CMF), свой 3-полосный и **расширенный 8-полосный** с графиком АЧХ.
- **Управление** — настройка жестов и кнопок.
- **Настройки устройства** — датчик ношения, двойное подключение со списком устройств, персональное шумоподавление, поиск наушников, тест посадки амбушюр, версия прошивки и серийный номер.
- **Пространственный звук**, **усиление басов**, **режим низкой задержки**, инструкции по сопряжению для каждой модели, автозапуск.

**Обновление прошивки** в приложение не входит: протокол обновлений и файлы прошивок Nothing закрыты, а неудачное
обновление может «окирпичить» наушники. Обновляйте их через Nothing X на телефоне — NothingX показывает текущую версию.

Все модели работают по общему протоколу Nothing; разработка и проверка велись на Headphone (1).

### Установка

1. Скачайте `NothingX-<версия>.dmg`, откройте и перетащите **NothingX** в **Программы**.
2. Приложение не нотаризовано Apple, поэтому при первом запуске macOS его заблокирует. Нажмите **«Готово»**,
   откройте **Системные настройки → Конфиденциальность и безопасность**, пролистайте вниз и нажмите
   **«Всё равно открыть»**, затем подтвердите паролем. Это нужно сделать один раз.
3. Разрешите доступ к Bluetooth. NothingX живёт в строке меню: если закрыть окно, приложение продолжит работать
   (выйти — кнопка **«Выйти»** в панели).

Наушники принимают только одно приложение управления — если управление не появляется, закройте Nothing X на телефоне.

Нужна macOS 13 или новее на Apple Silicon.

### Сборка

```bash
./scripts/package.sh   # dist/NothingX.app и dist/NothingX-<версия>.dmg
```
