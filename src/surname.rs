use phf;
use std::ascii::AsciiExt;
use super::namepart::NamePart;

static VOWELLESS_SURNAMES: [&'static str; 4] = ["Ng", "Lv", "Mtz", "Hdz"];

static SINGLE_LETTER_CONJUNCTIONS: [&'static str; 4] = ["e", "y", "E", "Y"];

// Uncapitalized list should match UNCAPITALIZED_PARTICLES in `namecase.rs`
//
// Unfortunately, the duplication between the lists is necessary because of the
// way `phf_set!` works, and the duplication within this list is necessary
// because we may or may not lowercase the particle as part of first-pass
// namecasing, depending on whether the input is given with mixed case.
static SURNAME_PREFIXES: phf::Set<&'static str> = phf_set! {
    "af",
    "av",
    "da",
    "das",
    "dal",
    "de",
    "del",
    "dela",
    "dei",
    "der",
    "di",
    "dí",
    "do",
    "dos",
    "du",
    "la",
    "le",
    "na",
    "ter",
    "van",
    "vel",
    "von",
    "zu",
    "zum",
    "Abu",
    "Abd",
    "Af",
    "Al",
    "Ap",
    "Av",
    "Aw",
    "Bar",
    "Ben",
    "Bon",
    "Bin",
    "Da",
    "Das",
    "Dal",
    "De",
    "Dei",
    "Del",
    "Dela",
    "Den",
    "Der",
    "Di",
    "Dí",
    "Do",
    "Dos",
    "Du",
    "El",
    "Ibn",
    "La",
    "Le",
    "Lo",
    "Na",
    "Nic",
    "San",
    "Santa",
    "St",
    "Ste",
    "Ter",
    "Van",
    "Vel",
    "Von",
    "Zu",
    "Zum",
};

pub fn is_vowelless_surname(word: &str, use_capitalization: bool) -> bool {
    if use_capitalization {
        VOWELLESS_SURNAMES.contains(&word)
    } else {
        VOWELLESS_SURNAMES.iter().any(|surname| surname.eq_ignore_ascii_case(word))
    }
}

pub fn find_surname_index(words: &[NamePart]) -> usize {
    if words.len() < 2 {
        return 0;
    }

    for (i, word) in words[0..words.len() - 1].iter().enumerate() {
        if SURNAME_PREFIXES.contains(&*word.namecased) {
            return i;
        }
    }

    for (i, word) in words[1..words.len() - 1].iter().enumerate() {
        if SINGLE_LETTER_CONJUNCTIONS.contains(&word.word) {
            // We found what looks like a conjunction in a Spanish or Portuguese
            // style surname (e.g. "Romero y Galdámez" or "Dato e Iradier"), so
            // the previous word was the start of the surname
            let orig_i = i + 1; // Index into original array
            if !words[orig_i - 1].is_initials() && !words[orig_i + 1].is_initials() {
                return orig_i - 1;
            }
        }
    }

    // Default case: just assume the last word is the surname
    words.len() - 1
}
