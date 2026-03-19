use crate::State;

fn xtime(x: u8) -> u8 {
    let mut val = x << 1;
    if x & 0x80 != 0 {
        val ^= 0x1b;
    }
    val
}

fn mul(x: u8, factor: u8) -> u8 {
    match factor {
        1 => x,
        2 => xtime(x),
        3 => xtime(x) ^ x,
        9 => xtime(xtime(xtime(x))) ^ x,
        11 => xtime(xtime(xtime(x))) ^ xtime(x) ^ x,
        13 => xtime(xtime(xtime(x))) ^ xtime(xtime(x)) ^ x,
        14 => xtime(xtime(xtime(x))) ^ xtime(xtime(x)) ^ xtime(x),
        _ => 0,
    }
}

pub(crate) fn mix_columns(state: State) -> State {
    const MATRIX: [[u8; 4]; 4] = [[2, 3, 1, 1], [1, 2, 3, 1], [1, 1, 2, 3], [3, 1, 1, 2]];

    let mut mixed = [[0u8; 4]; 4];
    for col in 0..4 {
        for row in 0..4 {
            mixed[col][row] = MATRIX[row]
                .iter()
                .enumerate()
                .fold(0, |acc, (k, factor)| acc ^ mul(state[col][k], *factor));
        }
    }
    mixed
}

pub(crate) fn inv_mix_columns(state: State) -> State {
    const MATRIX: [[u8; 4]; 4] = [
        [14, 11, 13, 9],
        [9, 14, 11, 13],
        [13, 9, 14, 11],
        [11, 13, 9, 14],
    ];

    let mut mixed = [[0u8; 4]; 4];
    for col in 0..4 {
        for row in 0..4 {
            mixed[col][row] = MATRIX[row]
                .iter()
                .enumerate()
                .fold(0, |acc, (k, factor)| acc ^ mul(state[col][k], *factor));
        }
    }
    mixed
}
