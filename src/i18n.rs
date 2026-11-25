//! Tiny i18n layer: Slint strings use `@tr()` + bundled .po files, Rust strings use `tr()`.

use std::sync::atomic::{AtomicBool, Ordering};

static RU: AtomicBool = AtomicBool::new(false);

/// System language (first preferred language) is Russian / Ukrainian / Belarusian.
pub fn system_is_ru() -> bool {
    let langs = objc2_foundation::NSLocale::preferredLanguages();
    langs.iter().next().map_or(false, |l| {
        let l = l.to_string().to_lowercase();
        l.starts_with("ru") || l.starts_with("uk") || l.starts_with("be")
    })
}

/// `pref`: "auto", "ru" or "en".
pub fn apply(pref: &str) {
    let ru = match pref {
        "ru" => true,
        "en" => false,
        _ => system_is_ru(),
    };
    RU.store(ru, Ordering::Relaxed);
    let _ = slint::select_bundled_translation(if ru { "ru" } else { "" });
}

pub fn ru() -> bool {
    RU.load(Ordering::Relaxed)
}

pub fn tr(en: &'static str) -> &'static str {
    if !ru() {
        return en;
    }
    match en {
        // EQ presets
        "Balanced" => "Баланс",
        "More bass" => "Больше баса",
        "More treble" => "Больше высоких",
        "Voice" => "Голос",
        "Custom" => "Свой",
        "Advanced" => "Продвинутый",
        "Rock" => "Рок",
        "Electronic" => "Электроника",
        "Pop" => "Поп",
        "Enhance vocals" => "Усиление вокала",
        "Classical" => "Классика",
        // connection
        "Reconnect" => "Подключиться снова",
        "Connect" => "Подключить",
        "Connecting…" => "Подключение…",
        "Control service not found" => "Служба управления не найдена",
        "Could not connect" => "Не удалось подключиться",
        "Busy: close Nothing X on your phone" => "Занято: закройте Nothing X на телефоне",
        "Not connected" => "Не подключено",
        "No device" => "Нет устройства",
        "Case" => "Кейс",
        "L" => "Л",
        "R" => "П",
        "C" => "К",
        // gestures: places
        "Left earbud" => "Левый наушник",
        "Right earbud" => "Правый наушник",
        "Case knob" => "Колесо на кейсе",
        "Button" => "Кнопка",
        "Roller" => "Колесо",
        "Slider" => "Слайдер",
        "Controls" => "Управление",
        // gestures: types
        "Press" => "Нажатие",
        "Double pinch" => "Двойное сжатие",
        "Triple pinch" => "Тройное сжатие",
        "Pinch & hold" => "Сжатие и удержание",
        "Double pinch & hold" => "Двойное сжатие и удержание",
        "Double tap" => "Двойное касание",
        "Triple tap" => "Тройное касание",
        "Tap & hold" => "Касание и удержание",
        "Double tap & hold" => "Двойное касание и удержание",
        "Double press" => "Двойное нажатие",
        "Triple press" => "Тройное нажатие",
        "Press & hold" => "Нажатие и удержание",
        "Double press & hold" => "Двойное нажатие и удержание",
        "Rotate" => "Вращение",
        "Slide" => "Сдвиг",
        // gestures: actions
        "No action" => "Нет действия",
        "Play / pause" => "Воспроизведение / пауза",
        "Previous track" => "Предыдущий трек",
        "Next track" => "Следующий трек",
        "Noise cancellation / transparency" => "Шумоподавление / прозрачность",
        "Noise cancellation / off" => "Шумоподавление / выкл.",
        "Transparency / off" => "Прозрачность / выкл.",
        "Noise control: all modes" => "Шумоподавление: все режимы",
        "Voice assistant" => "Голосовой ассистент",
        "Game mode" => "Игровой режим",
        "Volume up" => "Громкость +",
        "Volume down" => "Громкость −",
        "Volume" => "Громкость",
        "Take a photo" => "Сделать фото",
        "Spatial audio" => "Пространственный звук",
        "Microphone on / off" => "Микрофон вкл. / выкл.",
        "News" => "Новости",
        "Switch EQ preset" => "Смена пресета эквалайзера",
        "Treble boost" => "Усиление высоких",
        "Recording" => "Запись",
        "Unknown" => "Неизвестно",
        // noise levels
        "High" => "Высокое",
        "Mid" => "Среднее",
        "Low" => "Низкое",
        "Adaptive" => "Адаптивное",
        // dual connection / misc
        "Dual connection on" => "Двойное подключение вкл.",
        "Dual connection off" => "Двойное подключение выкл.",
        "Good fit" => "Хорошая посадка",
        "Try another ear tip" => "Попробуйте другой амбушюр",
        "Not measured" => "Не измерено",
        // about rows
        "Name" => "Название",
        "Model" => "Модель",
        "Firmware" => "Прошивка",
        "Serial number" => "Серийный номер",
        "Bluetooth address" => "Адрес Bluetooth",
        " · tested" => " · проверено",
        " · experimental" => " · экспериментально",
        _ => en,
    }
}
