use gtk::gio;
use gtk::prelude::*;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Chord {
    pub name: String,
    pub positions: Vec<[Option<usize>; 6]>,
}

pub fn load_chords() -> Vec<Chord> {
    let raw_data = gio::File::for_uri("resource:///dev/bragefuglseth/Fretboard/chords.json")
        .load_contents(None::<&gio::Cancellable>)
        .unwrap()
        .0;
    let json_string = std::str::from_utf8(&raw_data).unwrap();

    serde_json::from_str(json_string).unwrap()
}
