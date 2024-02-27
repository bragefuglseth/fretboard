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
        .collect();

    // Try to generate a simple chord name in the style of "Fmaj7"
    if let Some(name) = find_isolated_chord_name(&notes) {
        return Some(name);
    }

    let notes_without_first = notes.clone().into_iter().skip(1).collect::<Vec<usize>>();

    // Try to generate a chord name with an alternate bass note, in the style of "Fmaj7/C"
    if let Some(name) = find_isolated_chord_name(&notes_without_first) {
        let bass_note = find_note_name(*notes.first()?);

        println!("algorithm {name}/{bass_note}");

        return Some(format!("{name}/{bass_note}"));
    }

    None
}

fn find_note_name(value: usize) -> &'static str {
    match value {
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
    }
}

// Generates a chord name from a set of notes, e.g. "Am" or "Fmaj7", without taking alternative
// bass notes into account
fn find_isolated_chord_name(notes: &Vec<usize>) -> Option<String> {
    let first = notes.first()?;

    let intervals: Vec<usize> = notes
        .iter()
        .unique()
        .map(|n| sub_wrapping(*n, *first, SCALE_SIZE))
        .sorted()
        .collect();

    dbg!(&intervals);

    let note_name = find_note_name(*first);

    // this list can be extended with more possible intervals
    let suffix = match intervals.as_slice() {
        [0, 4, 7] => "",
        [0, 3, 7] => "m",
        [0, 3, 6] => "dim",
        [0, 7] => "5",
        [0, 4, 7, 11] => "maj7",
        [0, 3, 7, 11] => "mmaj7",
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
        [0, 4, 5, 7, 11] => "maj7add11",
        [0, 3, 5, 7, 11] => "mmaj7add11",
        [0, 4, 7, 9, 10] => "7add13",
        [0, 3, 7, 9, 10] => "m7add13",
        [0, 4, 7, 9, 11] => "maj7add13",
        [0, 3, 7, 9, 11] => "mmaj7add13",
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
