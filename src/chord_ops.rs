use itertools::Itertools;

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
    chord
        .iter()
        .filter_map(|&option| option)
        .filter(|&val| val > 0)
        .min()
        .map(|val| val.try_into().unwrap())
}

pub fn lowest_fingered_fret_special_casing(chord: [Option<usize>; 6]) -> Option<u8> {
    let lowest_fingered_fret = lowest_fingered_fret(chord);

    let adjusted_chord = adjust_chord(chord, lowest_fingered_fret.unwrap_or(0));

    match lowest_fingered_fret {
        Some(2) if find_barre_length(adjusted_chord) == 0 => Some(1),
        other => other,
    }
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
        .chain(input.chars().filter(|c| !c.is_ascii_whitespace()))
        .map(|c| c.to_ascii_lowercase())
        .tuple_windows::<(_, _)>()
        .map(|tuple| match tuple {
            ('a' | 'c' | 'd' | 'f' | 'g' | 'j' | 'm' | '2' | '4' | '7', '#') => '♯',
            ('a' | 'b' | 'd' | 'e' | 'g' | 'j' | 'm' | '2' | '4' | '7', 'b') => '♭',
            (' ' | '/', c) => c.to_ascii_uppercase(),
            (_, c) => c,
        })
        .collect()
}

pub fn serialize_chord_name(input: &str) -> String {
    input
        .replace("♯", "#")
        .replace("♭", "b")
        .to_ascii_lowercase()
}

pub fn enharmonic_equivalent(chord_name: &str) -> Option<&str> {
    match chord_name.get(0..2) {
        Some("c#") => Some("db"),
        Some("d#") => Some("eb"),
        Some("g#") => Some("ab"),
        Some("a#") => Some("bb"),

        Some("db") => Some("c#"),
        Some("eb") => Some("d#"),
        Some("ab") => Some("g#"),
        Some("bb") => Some("a#"),
        _ => None,
    }
}
