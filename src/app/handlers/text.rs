use crate::model::InputState;

pub(crate) const MAX_INPUT_LEN: usize = 8192;

pub(crate) fn push_char(input: &mut InputState, c: char) {
    if input.value.len() >= MAX_INPUT_LEN {
        return;
    }
    input.value.push(c);
    input.cursor += 1;
}

pub(crate) fn backspace(input: &mut InputState) {
    if input.cursor > 0 {
        input.value.remove(input.cursor - 1);
        input.cursor -= 1;
    }
}

pub(crate) const fn cursor_left(input: &mut InputState) {
    if input.cursor > 0 {
        input.cursor -= 1;
    }
}
