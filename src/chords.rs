use gtk::gio;
use gtk::prelude::*;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Chord {
    pub name: String,
    pub positions: Vec<[Option<usize>; 6]>,
}

pub fn load_chords() -> Vec<Chord> {
    let raw_data = gio::File::for_uri("resource:///dev/bragefuglseth/Fretboard/chords.txt")
        .load_contents(None::<&gio::Cancellable>)
        .unwrap()
        .0;
    let data_string = std::str::from_utf8(&raw_data).unwrap();
    let lines: Vec<&str> = data_string.lines().collect();
    let chunks = lines.split(|line| line.is_empty());

    let mut chords = Vec::new();

    for chunk in chunks {
        let mut chunk_iter = chunk.iter();
        let name = chunk_iter.next().unwrap();

        let positions: Vec<[Option<usize>; 6]> = chunk_iter
            .map(|line| {
                line.split(|c: char| c.is_whitespace())
                    .map(|c| match c {
                        "x" => None,
                        num => Some(num.parse::<usize>().unwrap()),
                    })
                    .collect::<Vec<_>>()
                    .try_into()
                    .unwrap()
            })
            .collect();

        chords.push(Chord {
            name: name.to_string(),
            positions,
        });
    }

    chords
}
