use servo::{
    Code, Key, KeyState, KeyboardEvent as ServoKeyboardEvent, Location, Modifiers,
    NamedKey as ServoNamedKey,
};
use winit::event::{ElementState, Modifiers as WinitModifiers};
use winit::keyboard::{Key as WinitKey, KeyCode, KeyLocation, NamedKey, PhysicalKey};

pub fn convert_key_event(
    event: &winit::event::KeyEvent,
    modifiers: &WinitModifiers,
) -> ServoKeyboardEvent {
    ServoKeyboardEvent::new_without_event(
        convert_state(event.state),
        convert_key(&event.logical_key),
        convert_code(&event.physical_key),
        convert_location(event.location),
        convert_modifiers(modifiers),
        event.repeat,
        false,
    )
}

fn convert_state(state: ElementState) -> KeyState {
    match state {
        ElementState::Pressed => KeyState::Down,
        ElementState::Released => KeyState::Up,
    }
}

fn convert_key(key: &WinitKey) -> Key {
    match key {
        WinitKey::Character(c) => Key::Character(c.to_string()),
        WinitKey::Named(named) => convert_named_key(*named),
        WinitKey::Unidentified(..) => Key::Named(ServoNamedKey::Unidentified),
        WinitKey::Dead(..) => Key::Named(ServoNamedKey::Unidentified),
    }
}

fn convert_named_key(key: NamedKey) -> Key {
    let named = match key {
        NamedKey::Enter => ServoNamedKey::Enter,
        NamedKey::Tab => ServoNamedKey::Tab,
        NamedKey::Space => return Key::Character(" ".into()),
        NamedKey::Backspace => ServoNamedKey::Backspace,
        NamedKey::Escape => ServoNamedKey::Escape,
        NamedKey::Delete => ServoNamedKey::Delete,
        NamedKey::ArrowDown => ServoNamedKey::ArrowDown,
        NamedKey::ArrowLeft => ServoNamedKey::ArrowLeft,
        NamedKey::ArrowRight => ServoNamedKey::ArrowRight,
        NamedKey::ArrowUp => ServoNamedKey::ArrowUp,
        NamedKey::End => ServoNamedKey::End,
        NamedKey::Home => ServoNamedKey::Home,
        NamedKey::PageDown => ServoNamedKey::PageDown,
        NamedKey::PageUp => ServoNamedKey::PageUp,
        NamedKey::CapsLock => ServoNamedKey::CapsLock,
        NamedKey::Shift => ServoNamedKey::Shift,
        NamedKey::Control => ServoNamedKey::Control,
        NamedKey::Alt => ServoNamedKey::Alt,
        NamedKey::Meta => ServoNamedKey::Meta,
        NamedKey::F1 => ServoNamedKey::F1,
        NamedKey::F2 => ServoNamedKey::F2,
        NamedKey::F3 => ServoNamedKey::F3,
        NamedKey::F4 => ServoNamedKey::F4,
        NamedKey::F5 => ServoNamedKey::F5,
        NamedKey::F6 => ServoNamedKey::F6,
        NamedKey::F7 => ServoNamedKey::F7,
        NamedKey::F8 => ServoNamedKey::F8,
        NamedKey::F9 => ServoNamedKey::F9,
        NamedKey::F10 => ServoNamedKey::F10,
        NamedKey::F11 => ServoNamedKey::F11,
        NamedKey::F12 => ServoNamedKey::F12,
        NamedKey::Insert => ServoNamedKey::Insert,
        NamedKey::ContextMenu => ServoNamedKey::ContextMenu,
        NamedKey::ScrollLock => ServoNamedKey::ScrollLock,
        NamedKey::NumLock => ServoNamedKey::NumLock,
        NamedKey::PrintScreen => ServoNamedKey::PrintScreen,
        NamedKey::Pause => ServoNamedKey::Pause,
        _ => ServoNamedKey::Unidentified,
    };
    Key::Named(named)
}

fn convert_code(physical: &PhysicalKey) -> Code {
    match physical {
        PhysicalKey::Code(code) => convert_key_code(*code),
        PhysicalKey::Unidentified(..) => Code::Unidentified,
    }
}

fn convert_key_code(code: KeyCode) -> Code {
    match code {
        KeyCode::Backquote => Code::Backquote,
        KeyCode::Backslash => Code::Backslash,
        KeyCode::BracketLeft => Code::BracketLeft,
        KeyCode::BracketRight => Code::BracketRight,
        KeyCode::Comma => Code::Comma,
        KeyCode::Digit0 => Code::Digit0,
        KeyCode::Digit1 => Code::Digit1,
        KeyCode::Digit2 => Code::Digit2,
        KeyCode::Digit3 => Code::Digit3,
        KeyCode::Digit4 => Code::Digit4,
        KeyCode::Digit5 => Code::Digit5,
        KeyCode::Digit6 => Code::Digit6,
        KeyCode::Digit7 => Code::Digit7,
        KeyCode::Digit8 => Code::Digit8,
        KeyCode::Digit9 => Code::Digit9,
        KeyCode::Equal => Code::Equal,
        KeyCode::KeyA => Code::KeyA,
        KeyCode::KeyB => Code::KeyB,
        KeyCode::KeyC => Code::KeyC,
        KeyCode::KeyD => Code::KeyD,
        KeyCode::KeyE => Code::KeyE,
        KeyCode::KeyF => Code::KeyF,
        KeyCode::KeyG => Code::KeyG,
        KeyCode::KeyH => Code::KeyH,
        KeyCode::KeyI => Code::KeyI,
        KeyCode::KeyJ => Code::KeyJ,
        KeyCode::KeyK => Code::KeyK,
        KeyCode::KeyL => Code::KeyL,
        KeyCode::KeyM => Code::KeyM,
        KeyCode::KeyN => Code::KeyN,
        KeyCode::KeyO => Code::KeyO,
        KeyCode::KeyP => Code::KeyP,
        KeyCode::KeyQ => Code::KeyQ,
        KeyCode::KeyR => Code::KeyR,
        KeyCode::KeyS => Code::KeyS,
        KeyCode::KeyT => Code::KeyT,
        KeyCode::KeyU => Code::KeyU,
        KeyCode::KeyV => Code::KeyV,
        KeyCode::KeyW => Code::KeyW,
        KeyCode::KeyX => Code::KeyX,
        KeyCode::KeyY => Code::KeyY,
        KeyCode::KeyZ => Code::KeyZ,
        KeyCode::Minus => Code::Minus,
        KeyCode::Period => Code::Period,
        KeyCode::Quote => Code::Quote,
        KeyCode::Semicolon => Code::Semicolon,
        KeyCode::Slash => Code::Slash,
        KeyCode::Backspace => Code::Backspace,
        KeyCode::CapsLock => Code::CapsLock,
        KeyCode::Enter => Code::Enter,
        KeyCode::Space => Code::Space,
        KeyCode::Tab => Code::Tab,
        KeyCode::Delete => Code::Delete,
        KeyCode::End => Code::End,
        KeyCode::Home => Code::Home,
        KeyCode::Insert => Code::Insert,
        KeyCode::PageDown => Code::PageDown,
        KeyCode::PageUp => Code::PageUp,
        KeyCode::ArrowDown => Code::ArrowDown,
        KeyCode::ArrowLeft => Code::ArrowLeft,
        KeyCode::ArrowRight => Code::ArrowRight,
        KeyCode::ArrowUp => Code::ArrowUp,
        KeyCode::Escape => Code::Escape,
        KeyCode::F1 => Code::F1,
        KeyCode::F2 => Code::F2,
        KeyCode::F3 => Code::F3,
        KeyCode::F4 => Code::F4,
        KeyCode::F5 => Code::F5,
        KeyCode::F6 => Code::F6,
        KeyCode::F7 => Code::F7,
        KeyCode::F8 => Code::F8,
        KeyCode::F9 => Code::F9,
        KeyCode::F10 => Code::F10,
        KeyCode::F11 => Code::F11,
        KeyCode::F12 => Code::F12,
        KeyCode::ShiftLeft => Code::ShiftLeft,
        KeyCode::ShiftRight => Code::ShiftRight,
        KeyCode::ControlLeft => Code::ControlLeft,
        KeyCode::ControlRight => Code::ControlRight,
        KeyCode::AltLeft => Code::AltLeft,
        KeyCode::AltRight => Code::AltRight,
        KeyCode::SuperLeft => Code::MetaLeft,
        KeyCode::SuperRight => Code::MetaRight,
        _ => Code::Unidentified,
    }
}

fn convert_location(location: KeyLocation) -> Location {
    match location {
        KeyLocation::Standard => Location::Standard,
        KeyLocation::Left => Location::Left,
        KeyLocation::Right => Location::Right,
        KeyLocation::Numpad => Location::Numpad,
    }
}

fn convert_modifiers(modifiers: &WinitModifiers) -> Modifiers {
    let state = modifiers.state();
    let mut m = Modifiers::empty();
    if state.shift_key() {
        m |= Modifiers::SHIFT;
    }
    if state.control_key() {
        m |= Modifiers::CONTROL;
    }
    if state.alt_key() {
        m |= Modifiers::ALT;
    }
    if state.super_key() {
        m |= Modifiers::META;
    }
    m
}
