use rayon::prelude::*;
use std::include_str;

#[derive(Debug)]
pub struct Chord {
    pub name: String,
    pub positions: Vec<[Option<usize>; 6]>,
}

pub struct ChordsDatabase(Vec<Chord>);

impl Default for ChordsDatabase {
    fn default() -> Self {
        Self::load()
    }
}

impl ChordsDatabase {
    pub fn load() -> Self {
        let data_string = include_str!("../data/chords.txt");
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

        ChordsDatabase(chords)
    }

    pub fn chord_from_name(&self, name: &str) -> Option<&Chord> {
        self.0
            .par_iter()
            .find_first(|chord| chord.name.to_lowercase() == name.to_lowercase())
    }

    pub fn name_from_chord(&self, query_chord: [Option<usize>; 6]) -> Option<String> {
        self.0
            .par_iter()
            .find_first(|chord| {
                chord
                    .positions
                    .par_iter()
                    .any(|&position| position == query_chord)
            })
            .map(|chord| chord.name.to_owned())
    }
}
