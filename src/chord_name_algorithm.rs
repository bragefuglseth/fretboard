// used as a fallback when a chord with a specific pattern can't be found in the database

use itertools::Itertools;

const SCALE_SIZE: usize = 12;

pub fn calculate_chord_name(chord: [Option<usize>; 6]) -> Option<String> {
    let notes: Vec<usize> = chord
        .iter()
        .enumerate()
        .filter_map(|tuplet| match tuplet {
            (0, Some(n)) => Some(n + 7),
            (1, Some(n)) => Some(n + 0),
            (2, Some(n)) => Some(n + 5),
            (3, Some(n)) => Some(n + 10),
            (4, Some(n)) => Some(n + 2),
            (5, Some(n)) => Some(n + 7),
            _ => None,
        })
        .map(|val| val % SCALE_SIZE)
        .unique()
        .collect();

    let Some(root) = notes.get(0) else { return None };

    let intervals: Vec<usize> = notes
        .iter()
        .map(|n| sub_wrapping(*n, *root, SCALE_SIZE))
        .sorted()
        .collect();

    let note_name = match root {
        0 => "A",
        1 => "A#",
        2 => "B",
        3 => "C",
        4 => "C#",
        5 => "D",
        6 => "D#",
        7 => "E",
        8 => "F",
        9 => "F#",
        10 => "G",
        11 => "G#",
        _ => panic!("root note above 11"),
    };

    // this list can be extended with more possible intervals
    let suffix = match intervals.as_slice() {
        [0, 4, 7] => "",
        [0, 3, 7] => "m",
        [0, 3, 6] => "dim",
        [0, 7] => "5",
        [0, 4, 7, 11] => "Δ7",
        [0, 3, 7, 11] => "mΔ7",
        [0, 4, 7, 10] => "7",
        [0, 3, 7, 10] => "m7",
        [0, 3, 6, 10] => "m7b5",
        [0, 3, 6, 9] => "dim7",
        [0, 2, 7] => "sus2",
        [0, 5, 7] => "sus4",
        [0, 4, 8] => "aug",
        [0, 4, 8, 11] => "augmaj11",
        [0, 4, 8, 10] => "aug7",
        [0, 2, 4, 7, 11] => "maj9",
        [0, 2, 3, 7, 10] => "m9",
        [0, 2, 4, 7, 10] => "9",
        [0, 2, 4, 5, 7, 11] => "maj11",
        [0, 2, 3, 5, 7, 10] => "m11",
        [0, 2, 4, 5, 7, 10] => "11",
        [0, 2, 4, 5, 7, 9, 10] => "13",
        [0, 2, 4, 7] => "add9",
        [0, 2, 3, 7] => "madd9",
        [0, 4, 5, 7] => "add11",
        [0, 3, 5, 7] => "madd11",
        [0, 4, 7, 9] => "add13",
        [0, 3, 7, 9] => "madd13",
        [0, 4, 5, 7, 10] => "7add11",
        [0, 3, 5, 7, 10] => "m7add11",
        [0, 4, 5, 7, 11] => "Δ7add11",
        [0, 3, 5, 7, 11] => "mΔ7add11",
        [0, 4, 7, 9, 10] => "7add13",
        [0, 3, 7, 9, 10] => "m7add13",
        [0, 4, 7, 9, 11] => "Δ7add13",
        [0, 3, 7, 9, 11] => "mΔ7add13",
        _ => return None,
    };

    Some(format!("{note_name}{suffix}"))
}

fn sub_wrapping(a: usize, b: usize, max: usize) -> usize {
    if a >= b {
        a - b
    } else {
        max - (b - a)
    }
}
