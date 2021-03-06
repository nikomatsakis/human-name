//! A library for parsing and comparing human names.
//!
//! See the documentation of the `Name` struct for details.

#![doc(html_root_url = "https://djudd.github.io/human-name/")]

#![feature(libc)]
#![feature(plugin)]
#![plugin(phf_macros)]

extern crate phf;
extern crate itertools;
extern crate unicode_segmentation;
extern crate unicode_normalization;
extern crate unidecode;
extern crate rustc_serialize;

#[macro_use]
mod utils;
mod suffix;
mod nickname;
mod title;
mod surname;
mod namecase;
mod namepart;
mod parse;
mod comparison;
mod serialization;
mod web_match;

pub mod external;

#[cfg(feature = "name_eq_hash")]
mod eq_hash;

use std::borrow::Cow;
use std::cell::Cell;
use std::hash::{Hash, Hasher, SipHasher};
use std::slice::Iter;
use std::str::Chars;
use std::iter::{Peekable, Enumerate};
use itertools::Itertools;
use utils::{is_mixed_case, transliterate, lowercase_if_alpha};

/// Represents a parsed human name.
///
/// Guaranteed to contain (what we think is) a surname, a first initial, and
/// nothing more. May also contain given & middle names, middle initials, and/or
/// a generational suffix.
///
/// Construct a Name using `parse`:
///
/// ```
/// use human_name::Name;
///
/// let name = Name::parse("Jane Doe").unwrap();
/// ```
///
/// Once you have a Name, you may extract is components, convert it to JSON,
/// or compare it with another Name to see if they are consistent with representing
/// the same person (see docs on `consistent_with` for details).
pub struct Name {
    words: Vec<String>,
    surname_index: usize,
    generation_from_suffix: Option<usize>,
    initials: String,
    word_indices_in_initials: Vec<(usize, usize)>,
    hash: Cell<Option<u64>>,
}

impl Name {

    /// Parses a string represent a single person's full name into a canonical
    /// representation.
    ///
    /// # Examples
    /// ```
    /// use human_name::Name;
    ///
    /// let name = Name::parse("Jane Doe").unwrap();
    /// assert_eq!("Doe", name.surname());
    /// assert_eq!(Some("Jane"), name.given_name());
    ///
    /// let name = Name::parse("Doe, J").unwrap();
    /// assert_eq!("Doe", name.surname());
    /// assert_eq!(None, name.given_name());
    /// assert_eq!('J', name.first_initial());
    ///
    /// let name = Name::parse("Dr. Juan Alberto T. Velasquez y Garcia III").unwrap();
    /// assert_eq!("Velasquez y Garcia", name.surname());
    /// assert_eq!(Some("Juan"), name.given_name());
    /// assert_eq!(Some("AT"), name.middle_initials());
    /// assert_eq!(Some("III"), name.suffix());
    /// ```
    ///
    /// # Supported formats
    ///
    /// Supports a variety of formats, including prefix and postfix titles,
    /// parenthesized nicknames, initials with and without periods, and sort
    /// order ("Doe, Jane"). Makes use of heuristics based on case when
    /// applicable (e.g., "AL Doe" is parsed as "A. L. Doe", while "Al Doe" is
    /// parsed as a given name and surname), as well as _small_ sets of known
    /// particles, conjunctions, titles, etc.
    ///
    /// # Limitations
    ///
    /// Errs on the side of producing parse output rather than giving up, so
    /// this function is _not_ suitable as a way of guessing whether a given
    /// string actually represents a name.
    ///
    /// However, success requires at least an apparent surname and first initial.
    /// Single-word names cannot be parsed (you may or may not wish to assume
    /// they are given names).
    ///
    /// Does not preserve titles (other than generational suffixes such as "III")
    /// or nicknames. Does not handle plural forms specially: "Mr. & Mrs. John
    /// Doe" will be parsed as "John Doe", and "Jane Doe, et al" will be parsed
    /// as "Jane Doe".
    ///
    /// Works best on Latin names - i.e., data from North or South America or
    /// Europe. Does not understand surname-first formats without commas: "Kim
    /// Il-sung" will be parsed as having the first name "Kim".
    ///
    /// Handles non-Latin unicode strings, but without any particular intelligence.
    /// Attempts at least to fail nicely, such that either `parse` returns `None`,
    /// or calling `display_full()` on the parsed result returns the input,
    /// plus or minus whitespace.
    ///
    /// Of course, [there is no perfect algorithm](http://www.kalzumeus.com/2010/06/17/falsehoods-programmers-believe-about-names/)
    /// for canonicalizing names. The goal here is to do the best we can without
    /// large statistical models.
    pub fn parse(name: &str) -> Option<Name> {
        if name.len() >= 1000 || !name.chars().any(char::is_alphabetic) {
            return None;
        }

        let mixed_case = is_mixed_case(name);
        let name = nickname::strip_nickname(name);

        let result = parse::parse(&*name, mixed_case);
        if result.is_none() {
            return None;
        }

        let (words, surname_index, generation_from_suffix) = result.unwrap();

        let mut names: Vec<String> = Vec::with_capacity(words.len());
        let mut initials = String::with_capacity(surname_index);
        let mut surname_index_in_names = surname_index;
        let mut word_indices_in_initials: Vec<(usize, usize)> = Vec::with_capacity(surname_index);

        for (i, word) in words.into_iter().enumerate() {
            if word.is_initials() && i < surname_index {
                initials.extend(word.namecased
                                    .chars()
                                    .filter(|c| c.is_alphabetic())
                                    .flat_map(|c| c.to_uppercase()));

                surname_index_in_names -= 1;
            } else if i < surname_index {
                let prior_len = initials.len();

                initials.extend(word.namecased
                                    .split('-')
                                    .filter_map(|w| w.chars().find(|c| c.is_alphabetic()))
                                    .flat_map(|c| c.to_uppercase()));

                names.push(word.namecased.into_owned());
                word_indices_in_initials.push((prior_len, initials.len()));
            } else {
                names.push(word.namecased.into_owned());
            }
        }

        debug_assert!(!names.is_empty(), "Names are empty!");
        //debug_assert!(initials.len() > 0, "Initials are empty!");
        //
        // This shouldn't be necessary, but we've found a case - "ﾟ." - where
        // the `is_alphabetic()` filter passes in `NamePart::from_word` but
        // fails in the `initials.extend` filter above, somehow.
        if initials.is_empty() {
            return None;
        }

        names.shrink_to_fit();
        word_indices_in_initials.shrink_to_fit();

        Some(Name {
            words: names,
            surname_index: surname_index_in_names,
            generation_from_suffix: generation_from_suffix,
            initials: initials,
            word_indices_in_initials: word_indices_in_initials,
            hash: Cell::new(None),
        })
    }

    /// First initial (always present)
    pub fn first_initial(&self) -> char {
        self.initials.chars().nth(0).unwrap()
    }

    /// Given name as a string, if present
    pub fn given_name(&self) -> Option<&str> {
        if self.surname_index > 0 {
            Some(&*self.words[0])
        } else {
            None
        }
    }

    /// Does this person use a middle name in place of their given name (e.g., T. Boone Pickens)?
    pub fn goes_by_middle_name(&self) -> bool {
        !self.word_indices_in_initials.is_empty() && self.word_indices_in_initials[0].0 > 0
    }

    /// First and middle initials as a string (always present)
    pub fn initials(&self) -> &str {
        &self.initials
    }

    /// Middle names as an array of words, if present
    pub fn middle_names(&self) -> Option<&[String]> {
        if self.surname_index > 1 {
            Some(&self.words[1..self.surname_index])
        } else {
            None
        }
    }

    /// Middle names as a string, if present
    pub fn middle_name(&self) -> Option<Cow<str>> {
        match self.middle_names() {
            Some(words) => {
                if words.len() == 1 {
                    Some(Cow::Borrowed(&*words[0]))
                } else {
                    Some(Cow::Owned(words.join(" ")))
                }
            }
            None => None,
        }
    }

    /// Middle initials as a string, if present
    pub fn middle_initials(&self) -> Option<&str> {
        match self.initials().char_indices().skip(1).nth(0) {
            Some((i, _)) => Some(&self.initials[i..]),
            None => None,
        }
    }

    /// Surname as a slice of words (always present)
    pub fn surnames(&self) -> &[String] {
        &self.words[self.surname_index..]
    }

    /// Surname as a string (always present)
    pub fn surname(&self) -> Cow<str> {
        if self.surnames().len() > 1 {
            Cow::Owned(self.surnames().join(" "))
        } else {
            Cow::Borrowed(&*self.surnames()[0])
        }
    }

    /// Generational suffix, if present
    pub fn suffix(&self) -> Option<&str> {
        self.generation_from_suffix.map(|g| suffix::display_generational_suffix(g))
    }

    fn given_names_or_initials(&self) -> GivenNamesOrInitials {
        GivenNamesOrInitials {
            initials: self.initials.chars().enumerate(),
            known_names: self.words[0..self.surname_index].iter(),
            known_name_indices: self.word_indices_in_initials.iter().peekable(),
        }
    }

    /// First initial (with period) and surname.
    ///
    /// ```
    /// use human_name::Name;
    ///
    /// let name = Name::parse("JOHN ALLEN Q DE LA MACDONALD JR").unwrap();
    /// assert_eq!("J. de la MacDonald", name.display_initial_surname());
    /// ```
    pub fn display_initial_surname(&self) -> String {
        format!("{}. {}", self.first_initial(), self.surname())
    }

    /// Given name and surname, if given name is known, otherwise first initial
    /// and surname.
    ///
    /// ```
    /// use human_name::Name;
    ///
    /// let name = Name::parse("JOHN ALLEN Q DE LA MACDONALD JR").unwrap();
    /// assert_eq!("John de la MacDonald", name.display_first_last());
    /// ```
    pub fn display_first_last(&self) -> String {
        match self.given_name() {
            Some(ref name) => {
                format!("{} {}", name, self.surname())
            }
            None => {
                self.display_initial_surname()
            }
        }
    }

    /// Number of bytes in the full name as UTF-8 in NFKD normal form, including
    /// spaces and punctuation.
    ///
    /// ```
    /// use human_name::Name;
    ///
    /// let short_name = Name::parse("John Doe").unwrap();
    /// assert_eq!("John Doe".len(), short_name.byte_len());
    ///
    /// let long_name = Name::parse("JOHN ALLEN Q DE LA MACDÖNALD JR").unwrap();
    /// assert_eq!("John Allen Q. de la MacDönald, Jr.".len(), long_name.byte_len());
    /// ```
    pub fn byte_len(&self) -> usize {
        // Words plus spaces
        let mut len = self.words
                          .iter()
                          .fold(self.words.len() - 1, |sum, ref word| sum + word.len());

        if let Some(suffix) = self.suffix() {
            len += 2; // Comma and space
            len += suffix.len();
        }

        let extra_initials = self.initials.chars().count() - self.surname_index;
        if extra_initials > 0 {
            len += self.initials.len() -
                   self.words[0..self.surname_index]
                       .iter()
                       .fold(0, |sum, ref word| sum + word.chars().nth(0).unwrap().len_utf8());

            len += 2 * extra_initials; // Period and space for each initial
        }

        len
    }

    /// The full name, or as much of it as was preserved from the input,
    /// including given name, middle names, surname and suffix.
    ///
    /// ```
    /// use human_name::Name;
    ///
    /// let name = Name::parse("JOHN ALLEN Q DE LA MACDONALD JR").unwrap();
    /// assert_eq!("John Allen Q. de la MacDonald, Jr.", name.display_full());
    /// ```
    pub fn display_full(&self) -> String {
        let mut result = String::with_capacity(self.byte_len());

        for part in self.given_names_or_initials() {
            match part {
                NameWordOrInitial::Word(name, _) => {
                    result.push_str(name);
                    result.push(' ');
                }
                NameWordOrInitial::Initial(initial) => {
                    result.push(initial);
                    result.push_str(". ");
                }
            }
        }

        let surnames = self.surnames();
        if surnames.len() > 1 {
            for word in surnames[0..surnames.len() - 1].iter() {
                result.push_str(word);
                result.push(' ');
            }
        }
        result.push_str(&surnames[surnames.len() - 1]);

        if let Some(suffix) = self.suffix() {
            result.push_str(", ");
            result.push_str(suffix);
        }

        result
    }

    /// Implements a hash for a name that is always identical for two names that
    /// may be consistent according to our matching algorithm.
    ///
    /// ### WARNING
    ///
    /// This hash function is prone to collisions!
    ///
    /// We can only use the last four alphabetical characters of the surname,
    /// because that's all we're guaranteed to use in the consistency test. That
    /// means if names are ASCII, we only have 19 bits of variability.
    ///
    /// That means if you are working with a lot of names and you expect surnames
    /// to be similar or identical, you might be better off avoiding hash-based
    /// datastructures (or using a custom hash and matching algorithm).
    ///
    /// We can't use more characters of the surname because we treat names as equal
    /// when one surname ends with the other and the smaller is at least four
    /// characters, to catch cases like "Iria Gayo" == "Iria del Río Gayo".
    ///
    /// We can't use the first initial because we might ignore it if someone goes
    /// by a middle name or nickname, or due to transliteration.
    pub fn surname_hash<H: Hasher>(&self, state: &mut H) {
        let surname_chars = self.surnames()
                                .iter()
                                .flat_map(|w| w.chars())
                                .flat_map(transliterate)
                                .rev();
        for c in surname_chars.filter_map(lowercase_if_alpha)
                              .take(comparison::MIN_SURNAME_CHAR_MATCH) {
            c.hash(state);
        }
    }

    /// Memoizes the result of `surname_hash` when used with `SipHasher`
    pub fn memoized_surname_hash(&self) -> u64 {
        {
            let cached = self.hash.get();
            if cached.is_some() {
                return cached.unwrap();
            }

            let mut s = SipHasher::new();
            self.surname_hash(&mut s);
            self.hash.set(Some(s.finish()));
        }

        self.memoized_surname_hash()
    }
}

struct GivenNamesOrInitials<'a> {
    initials: Enumerate<Chars<'a>>,
    known_names: Iter<'a, String>,
    known_name_indices: Peekable<Iter<'a, (usize, usize)>>,
}

#[derive(Debug)]
enum NameWordOrInitial<'a> {
    Word(&'a str, usize),
    Initial(char),
}

impl <'a>Iterator for GivenNamesOrInitials<'a> {
    type Item = NameWordOrInitial<'a>;

    fn next(&mut self) -> Option<NameWordOrInitial<'a>> {
        match self.initials.next() {
            Some((i, initial)) => {
                let mut next_name = None;
                let mut initials_for_word = 1;
                if let Some(&&(j, k)) = self.known_name_indices.peek() {
                    if j == i {
                        self.known_name_indices.next();
                        next_name = self.known_names.next();

                        // Handle case of hyphenated name for which we have 2+ initials
                        for _ in j + 1..k {
                            self.initials.next();
                            initials_for_word += 1;
                        }
                    }
                }

                if let Some(name) = next_name {
                    Some(NameWordOrInitial::Word(name, initials_for_word))
                } else {
                    Some(NameWordOrInitial::Initial(initial))
                }
            }
            None => None,
        }
    }
}
