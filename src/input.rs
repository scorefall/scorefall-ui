/// State of a key.
#[repr(u8)]
#[derive(PartialEq, Copy, Clone)]
pub enum KeyState {
    /// Not pressed
    Idle = 0b_00,
    /// Just got pressed
    Just = 0b_11,
    /// Held down.
    Held = 0b_10,
    /// Just got released
    Lift = 0b_01,
}

impl KeyState {
    /// Returns true if the key is held down.
    #[inline(always)]
    pub fn held(self) -> bool {
        (self as u8) >> 1 != 0
    }

    /// Returns true if the key was just pressed.
    #[inline(always)]
    pub fn press(self) -> bool {
        self == KeyState::Just
    }
}

/// A key on the Keyboard.
#[repr(usize)]
pub enum KeyName {
    Left = 0usize,
    Right,
    Up,
    Down,
    Max,
}

/// Input state.
pub struct InputState {
    /// Left Arrow Key
    pub keys: [KeyState; KeyName::Max as usize],
    /// If any Just or Lift has happened.
    pub has_input: bool,
    /// A unicode character.
    pub text: char,
}

impl InputState {
    /// Create a new keys / buttons state.
    pub fn new() -> InputState {
        InputState {
            keys: [KeyState::Idle; KeyName::Max as usize],
            has_input: false,
            text: '\0',
        }
    }

    /// Turn Just & Lift into Held & Idle
    pub fn reset(&mut self) {
        for key in &mut self.keys {
            *key = match *key {
                KeyState::Just => KeyState::Held,
                KeyState::Lift => KeyState::Idle,
                a => a,
            };
        }
        self.has_input = false;
    }

    pub fn update(&mut self, key: String, code: String, ic: bool, held: bool) {
        let state = if held { KeyState::Just } else { KeyState::Lift };
        self.has_input = true;

        // A unicode character has been typed.
        match code.as_str() {
            "Numpad0" => {}
            "Numpad1" => {}
            "Numpad2" => {}
            "Numpad3" => {}
            "Numpad4" => {}
            "Numpad5" => {}
            "Numpad6" => {}
            "Numpad7" => {}
            "Numpad8" => {}
            "Numpad9" => {}
            "AltLeft" | "AltRight" => {}
            "ControlLeft" | "ControlRight" => {}
            "Space" => {}
            "Tab" => {}
            "Backspace" => {}
            "Escape" => {}
            "Enter" => {}
            "NumpadEnter" => {}
            "ArrowUp" => self.keys[KeyName::Up as usize] = state,
            "ArrowDown" => self.keys[KeyName::Down as usize] = state,
            "ArrowLeft" => self.keys[KeyName::Left as usize] = state,
            "ArrowRight" => self.keys[KeyName::Right as usize] = state,
            _ => {
                if key.len() == 1 && !ic {
                    self.text = key.chars().nth(0).unwrap();
                }
            }
        }
    }
}
