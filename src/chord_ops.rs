use itertools::Itertools;

// These are always shown in fret position 1
const SPECIAL_CASE_CHORDS: [[Option<usize>; 6]; 20] = [
    [None, Some(0), Some(2), Some(2), Some(2), Some(0)], // A
    [None, Some(0), Some(2), Some(2), None, Some(0)],    // A5
    [None, Some(0), Some(2), Some(0), Some(2), Some(0)], // A7
    [None, Some(0), Some(2), Some(4), Some(2), Some(0)], // Aadd9
    [None, Some(0), Some(2), Some(2), Some(3), Some(0)], // Asus4
    [None, Some(2), Some(0), Some(2), Some(0), Some(2)], // Bm7
    [None, Some(3), Some(2), Some(0), Some(0), Some(0)], // Cmaj7
    [None, None, Some(0), Some(2), Some(3), Some(2)],    // D
    [None, None, Some(0), Some(2), Some(3), None],       // D5
    [None, None, Some(0), Some(2), Some(0), Some(2)],    // D6
    [None, None, Some(0), Some(2), Some(2), Some(2)],    // Dmaj7
    [None, None, Some(0), Some(2), Some(3), Some(3)],    // Dsus4
    [Some(0), Some(2), Some(2), Some(0), Some(0), Some(0)], // Em
    [Some(0), Some(2), Some(2), Some(0), Some(2), Some(0)], // Em6
    [Some(0), Some(2), Some(0), Some(0), Some(0), Some(0)], // Em7
    [Some(0), Some(2), Some(2), None, Some(0), Some(2)], // Esus2
    [Some(0), Some(2), Some(2), Some(2), Some(0), Some(0)], // Esus4
    [Some(3), Some(2), Some(0), Some(0), Some(0), Some(3)], // G
    [Some(3), Some(2), Some(0), Some(0), Some(3), Some(3)], // G alternative
    [Some(3), Some(2), Some(0), Some(0), Some(0), Some(0)], // G6
];

// find barre length of *adjusted* chords (lowest fingered fret is positioned @ 1)
pub fn find_barre_length(chord: [Option<usize>; 6]) -> usize {
    if chord
        .iter()
        .filter(|&&option| option == Some(1_usize))
        .count()
        .lt(&2)
    {
        return 0;
    }

    let mut barre_length = 0;

    let chord_reversed = chord.iter().rev().enumerate();

    let mut chord_reversed_next = chord.iter().rev();
    chord_reversed_next.next();

    let mut note_count = 0;

    for (num, val) in chord_reversed {
        if val == &Some(1_usize) {
            barre_length = num + 1;
            note_count += 1;
        }

        let next = chord_reversed_next.next();
        if next == Some(&Some(0_usize))
            || next == Some(&None)
            || val == &Some(0_usize)
            || val.is_none()
        {
            break;
        }
    }
    if note_count > 1 {
        barre_length
    } else {
        0
    }
}

pub fn lowest_fingered_fret(chord: [Option<usize>; 6]) -> Option<u8> {
    if SPECIAL_CASE_CHORDS
        .iter()
        .any(|&special_chord| special_chord == chord)
    {
        return Some(1);
    }

    chord
        .iter()
        .filter_map(|&option| option)
        .filter(|&val| val > 0)
        .min()
        .map(|val| val as u8)
}

pub fn adjust_chord(chord: [Option<usize>; 6], barre: u8) -> [Option<usize>; 6] {
    chord
        .iter()
        .map(|option| {
            option.map(|value| {
                if value == 0 {
                    0
                } else {
                    value - (if barre == 0 { barre } else { barre - 1 }) as usize
                }
            })
        })
        .collect::<Vec<Option<usize>>>()
        .try_into()
        .unwrap()
}

pub fn prettify_chord_name(input: &str) -> String {
    std::iter::once(' ')
        .chain(input.chars().filter(|c| !c.is_whitespace()))
        .map(|c| c.to_ascii_lowercase())
        .tuple_windows::<(_, _)>()
        .map(|tuple| match tuple {
            ('a' | 'c' | 'd' | 'f' | 'g' | 'j' | 'm' | '2' | '4' | '7', '#') => '♯',
            ('a' | 'b' | 'd' | 'e' | 'g' | 'j' | 'm' | '2' | '4' | '7', 'b') => '♭',
            (' ' | '/', c) => c.to_ascii_uppercase(),
            (_, c) => c,
        })
        .collect::<String>()
        .replace("dim", " dim")
        .replace("aug", " aug")
        .replace("maj", " maj")
}

pub fn serialize_chord_name(input: &str) -> String {
    input
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect::<String>()
        .replace('♯', "#")
        .replace('♭', "b")
        .to_ascii_lowercase()
}

pub fn enharmonic_equivalent(chord_name: &str) -> Option<&str> {
    match chord_name.get(0..2) {
        Some("c#") => Some("db"),
        Some("d#") => Some("eb"),
        Some("f#") => Some("gb"),
        Some("g#") => Some("ab"),
        Some("a#") => Some("bb"),

        Some("db") => Some("c#"),
        Some("eb") => Some("d#"),
        Some("gb") => Some("f#"),
        Some("ab") => Some("g#"),
        Some("bb") => Some("a#"),
        _ => None,
    }
}
