use crate::State;

pub(crate) fn shift_rows(state: State) -> State {
    let mut shifted = [[0u8; 4]; 4];
    for c in 0..4 {
        for r in 0..4 {
            shifted[c][r] = state[(c + r) % 4][r];
        }
    }
    shifted
}

pub(crate) fn inv_shift_rows(state: State) -> State {
    let mut shifted = [[0u8; 4]; 4];
    for c in 0..4 {
        for r in 0..4 {
            shifted[c][r] = state[(4 + c - r) % 4][r];
        }
    }
    shifted
}
