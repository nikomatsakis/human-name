use std::ascii::AsciiExt;
use std::borrow::Cow;
use super::utils::*;
use super::nickname::have_matching_variants;
use super::{Name, NameWordOrInitial};
use unicode_segmentation::UnicodeSegmentation;

pub const MIN_SURNAME_CHAR_MATCH: usize = 4;
pub const MIN_GIVEN_NAME_CHAR_MATCH: usize = 3;


impl Name {

    /// Might this name represent the same person as another name?
    ///
    /// # Examples
    /// ```
    /// use human_name::Name;
    ///
    /// let j_doe = Name::parse("J. Doe").unwrap();
    /// let jane_doe = Name::parse("Jane Doe").unwrap();
    /// let john_m_doe = Name::parse("John M. Doe").unwrap();
    /// let john_l_doe = Name::parse("John L. Doe").unwrap();
    ///
    /// assert!(j_doe.consistent_with(&john_m_doe));
    /// assert!(j_doe.consistent_with(&john_l_doe));
    /// assert!(j_doe.consistent_with(&jane_doe));
    /// assert!(j_doe.consistent_with(&j_doe));
    /// assert!(!john_m_doe.consistent_with(&john_l_doe));
    /// assert!(!jane_doe.consistent_with(&john_l_doe));
    ///
    /// let zheng_he = Name::parse("Zheng He").unwrap();
    /// let han_chars = Name::parse("鄭和").unwrap();
    /// assert!(han_chars.consistent_with(&zheng_he));
    /// ```
    ///
    /// # Defining "consistency"
    ///
    /// Requires that all known parts are consistent, which means at minimum,
    /// the final words of the surnames match, and one ordered set of first
    /// and middle initials is a superset of the other. If given and/or middle
    /// names and/or suffixes are present in both names, they must match as well.
    ///
    /// Transliterates everything to ASCII before comparison using the naive
    /// algorithm of [unidecode](https://github.com/chowdhurya/rust-unidecode/)
    /// (which ignores context), and ignores case, accents and combining marks.
    ///
    /// In the case of given and middle names, allows one name to be a prefix of
    /// the other, without requiring the prefix end at a word boundary as we do
    /// with surname suffix matches, and supports matching a small number of
    /// common nicknames and nickname patterns based on the root name.
    ///
    /// # Limitations
    ///
    /// There will be false positives ("Jan Doe" is probably not "Jane Doe"),
    /// and false negatives ("James Hanson" might be "James Hansen"). And, of
    /// course, even identical names do not necessarily represent the same person.
    ///
    /// Given limited information, we err on the side of false positives. This
    /// kind of matching will be most useful in cases where we already have
    /// reason to believe that a single individual's name appears twice, and we
    /// are trying to figure out exactly where, e.g. a particular author's index
    /// in the list of authors of a co-authored paper.
    ///
    #[cfg_attr(rustfmt, rustfmt_skip)]
    pub fn consistent_with(&self, other: &Name) -> bool {
        // Fast path
        if self.memoized_surname_hash() != other.memoized_surname_hash() {
            return false;
        }

        // Check given name(s) first because if we got this far, we know that
        // at least the last characters of the surnames are consistent
        self.given_and_middle_names_consistent(other) &&
        self.surname_consistent(other) &&
        self.suffix_consistent(other)
    }

    fn given_and_middle_names_consistent(&self, other: &Name) -> bool {
        // Handle simple cases first, where we only have to worry about one name
        // and/or initial.
        if self.middle_initials().is_none() && other.middle_initials().is_none() {
            if self.given_name().is_none() || other.given_name().is_none() {
                return to_ascii_letter(self.first_initial()) ==
                       to_ascii_letter(other.first_initial());
            } else {
                return have_matching_variants(self.given_name().unwrap(),
                                              other.given_name().unwrap());
            }
        }

        // For the more complicated cases, we'll simplify things a bit by
        // letting ourselves assume `self` has the more complete name.
        if self.initials().chars().count() >= other.initials().chars().count() {
            self.given_and_middle_names_consistent_with_less_complete(other)
        } else {
            other.given_and_middle_names_consistent_with_less_complete(self)
        }
    }

    fn given_and_middle_names_consistent_with_less_complete(&self, other: &Name) -> bool {
        // Check initials first
        if !self.initials_consistent_with_less_complete(other) {
            return false;
        }

        // Unless both versions of the name have given or middle names, we're done
        if self.surname_index == 0 || other.surname_index == 0 {
            return true;
        }

        // In the case where we have to compare multiple given or middle names,
        // align words using their initials, and for each initial where we know
        // the word for both names, transliterate to ASCII lowercase, then require
        // that the words are an exact match, or that one is a prefix of the other,
        // or that one is a recognized nickname or spelling variant of the other.

        let missing_any_names = self.missing_any_name() || other.missing_any_name();

        let mut their_parts = other.given_names_or_initials().peekable();
        let mut suffix_for_prior_prefix_match: Option<String> = None;
        let mut looked_up_nicknames = false;

        let mut their_part_if_any = their_parts.next();

        for my_part in self.given_names_or_initials() {
            if let Some(ref their_part) = their_part_if_any {
                let result = my_part.check_consistency(their_part, !looked_up_nicknames);

                match result {
                    ComparisonResult::Inconsistent => {
                        // The names are inconsistent
                        return false;
                    }
                    ComparisonResult::DifferentInitials => {
                        // They don't have a word for this initial, so we don't
                        // advance the iterator for their words/initials
                        continue;
                    }
                    ComparisonResult::NicknameMatch => {
                        looked_up_nicknames = true;
                    }
                    ComparisonResult::PrefixOfOther(remaining_chars) => {
                        suffix_for_prior_prefix_match = Some(remaining_chars);
                    }
                    _ => {
                        // Any other kind of match; no-op, just continue
                    }
                }
            } else if missing_any_names {
                // We've matched everything available, and will skip the check
                // in the next block
                return true;
            } else if let Some(suffix) = suffix_for_prior_prefix_match {
                // We've matched everything available, but we're not quite done.
                //
                // It's not uncommon for representations of a name to be inconsistent
                // in whether two parts of a given name are separated by a space,
                // a hyphen, or nothing (especially with transliterated names).
                //
                // This is one of the reasons we accept prefix-only matches for
                // given & middle names. However, doing so potentially opens us up
                // to more false positives, and we want to mitigate that.
                //
                // When it looks like we have a name part following a space or hyphen
                // which might reasonably be treated as part of the same actual name
                // as the preceding part, we don't want to just accept a prefix-only
                // match on the preceding part as sufficient for a full match.
                // Instead we'll continue after the prefix match by comparing the
                // following part to the suffix, just as if the two parts hadn't
                // been separated by a space or hyphen.
                //
                // This separates, e.g., "Jinli" from "Jin Yi", or "Xiaofeng" from
                // "Xiao Peng".
                //
                // We don't try to do this in the presence of middle initials
                // without corresponding names, because the logic would have to
                // be even more complicated.
                if let NameWordOrInitial::Word(word, _) = my_part {
                    return eq_or_starts_with!(suffix, word);
                } else {
                    return true;
                }
            } else {
                // We've matched everything available
                return true;
            }

            let mut advance_by = my_part.initials_count();
            while advance_by > 0 && their_part_if_any.is_some() {
                their_part_if_any = their_parts.next();
                if let Some(ref their_part) = their_part_if_any {
                    advance_by -= their_part.initials_count();
                }
            }
        }

        their_part_if_any.is_none()
    }

    fn initials_consistent_with_less_complete(&self, other: &Name) -> bool {
        let my_initials = &*self.transliterated_initials();
        let their_initials = &*other.transliterated_initials();

        // Unless we have both given names, we require that the first initials
        // match (if we do have the names, we'll check for nicknames, etc,
        // which might violate this requirement.)
        //
        // If we do have both, we skip this requirement because we might find
        // a nickname match.
        if self.missing_given_name() || other.missing_given_name() {
            if self.goes_by_middle_name() {
                // Edge case: if we go by a middle name, another version of the
                // name might use the middle name's initial as the first initial.
                //
                // However, only check in this direction because we know `self`
                // has more complete initials, so their initials won't contain
                // ours unless they're equal.
                if !my_initials.contains(&their_initials) {
                    return false;
                }
            } else {
                // Normal case, given the absence of (true) given names
                if my_initials.chars().nth(0) != their_initials.chars().nth(0) {
                    return false;
                }
            }
        }

        // If we have middle initials, check their consistency alone before
        // looking at names (& we know "self" has the more complete initials).
        //
        // Using byte offsets is ok because we already converted to ASCII.
        if my_initials.len() > 1 && their_initials.len() > 1 &&
           !my_initials[1..].contains(&their_initials[1..]) {
            return false;
        }

        true
    }

    fn transliterated_initials(&self) -> Cow<str> {
        if self.initials().is_ascii() {
            Cow::Borrowed(self.initials())
        } else {
            Cow::Owned(self.initials()
                           .chars()
                           .filter_map(to_ascii_letter)
                           .collect::<String>())
        }
    }

    fn missing_given_name(&self) -> bool {
        self.given_name().is_none() || self.goes_by_middle_name()
    }

    fn missing_any_name(&self) -> bool {
        if self.surname_index == 0 {
            return true;
        }

        let mut prev = 0;

        for &(i, j) in self.word_indices_in_initials.iter() {
            if i > prev {
                return true;
            } else {
                prev = j;
            }
        }

        self.surname_index > prev
    }

    fn surname_consistent(&self, other: &Name) -> bool {
        // Fast path
        if self.simple_surname() && other.simple_surname() {
            return self.surname().eq_ignore_ascii_case(&*other.surname());
        }

        let mut my_words = self.surnames()
                               .iter()
                               .flat_map(|w| w.unicode_words())
                               .rev();

        let mut their_words = other.surnames()
                                   .iter()
                                   .flat_map(|w| w.unicode_words())
                                   .rev();

        let mut my_word = my_words.next();
        let mut their_word = their_words.next();
        let mut matching_chars = 0;

        // Require either an exact match (ignoring case etc), or a partial match
        // of len >= MIN_SURNAME_CHAR_MATCH and breaking on a word boundary
        loop {
            // No words remaining for some surname - that's ok if it's true of
            // both, or if the components that match are long enough
            if my_word.is_none() && their_word.is_none() {
                return true;
            } else if my_word.is_none() || their_word.is_none() {
                return matching_chars >= MIN_SURNAME_CHAR_MATCH;
            }

            macro_rules! reverse_lowercase_alpha_chars {
                ($word:expr) => {
                    $word.unwrap().chars().flat_map(transliterate).rev().filter_map(lowercase_if_alpha)
                }
            }

            let mut my_chars = reverse_lowercase_alpha_chars!(my_word);
            let mut their_chars = reverse_lowercase_alpha_chars!(their_word);

            let mut my_char = my_chars.next();
            let mut their_char = their_chars.next();

            loop {
                if my_char.is_none() && their_char.is_none() {
                    // The words matched exactly, try the next word
                    my_word = my_words.next();
                    their_word = their_words.next();
                    break;
                } else if my_char.is_none() {
                    // My word is a suffix of their word, check my next word
                    // against the rest of their word
                    my_word = my_words.next();
                    if my_word.is_none() {
                        // There is no next word, so this is a suffix-only match,
                        // and we don't allow those
                        return false;
                    } else {
                        // Continue the inner loop but incrementing through my
                        // next word
                        my_chars = reverse_lowercase_alpha_chars!(my_word);
                        my_char = my_chars.next();
                    }
                } else if their_char.is_none() {
                    // Their word is a suffix of my word, check their next word
                    // against the rest of my_words
                    their_word = their_words.next();
                    if their_word.is_none() {
                        // There is no next word, so this is a suffix-only match,
                        // and we don't allow those
                        return false;
                    } else {
                        // Continue the inner loop but incrementing through their
                        // next word
                        their_chars = reverse_lowercase_alpha_chars!(their_word);
                        their_char = their_chars.next();
                    }
                } else if my_char != their_char {
                    // We found a conflict and can short-circuit
                    return false;
                } else {
                    // Characters matched, continue the inner loop
                    matching_chars += 1;
                    my_char = my_chars.next();
                    their_char = their_chars.next();
                }
            }
        }
    }

    fn simple_surname(&self) -> bool {
        self.surnames().len() == 1 && self.surname().chars().all(is_ascii_alphabetic)
    }

    fn suffix_consistent(&self, other: &Name) -> bool {
        self.generation_from_suffix.is_none() || other.generation_from_suffix.is_none() ||
        self.generation_from_suffix == other.generation_from_suffix
    }
}

#[derive(Eq,PartialEq,Debug)]
enum ComparisonResult {
    Inconsistent,
    DifferentInitials,
    InitialsOnlyMatch,
    ExactMatch,
    PrefixOfOther(String),
    PrefixOfSelf(String),
    NicknameMatch,
}

impl<'a> NameWordOrInitial<'a> {
    pub fn initial(&self) -> Option<char> {
        match self {
            &NameWordOrInitial::Word(word, _) => {
                word.chars().nth(0).and_then(|c| to_ascii_letter(c))
            }
            &NameWordOrInitial::Initial(initial) => {
                to_ascii_letter(initial)
            }
        }
    }

    pub fn check_consistency(&self,
                             other: &NameWordOrInitial,
                             allow_nicknames: bool)
                             -> ComparisonResult {
        if self.initial().is_none() || self.initial() != other.initial() {
            return ComparisonResult::DifferentInitials;
        }

        if !self.has_word() || !other.has_word() {
            return ComparisonResult::InitialsOnlyMatch;
        }

        let mut my_chars = self.word()
                               .chars()
                               .flat_map(transliterate)
                               .filter_map(lowercase_if_alpha);
        let mut their_chars = other.word()
                                   .chars()
                                   .flat_map(transliterate)
                                   .filter_map(lowercase_if_alpha);
        let mut matched = 0;

        loop {
            let my_char = my_chars.next();
            let their_char = their_chars.next();

            if my_char.is_none() && their_char.is_none() {
                return ComparisonResult::ExactMatch;
            } else if (my_char.is_none() || their_char.is_none()) && matched >= MIN_GIVEN_NAME_CHAR_MATCH {
                if their_char.is_some() {
                    return ComparisonResult::PrefixOfOther(format!("{}{}", their_char.unwrap(), their_chars.collect::<String>()));
                } else {
                    return ComparisonResult::PrefixOfSelf(format!("{}{}",
                                                                  my_char.unwrap(),
                                                                  my_chars.collect::<String>()));
                }
            } else if my_char != their_char {
                // Failed match; abort, but first, maybe try nickname db
                if allow_nicknames && have_matching_variants(self.word(), other.word()) {
                    return ComparisonResult::NicknameMatch;
                } else {
                    return ComparisonResult::Inconsistent;
                }
            } else {
                matched += 1;
            }
        }
    }

    fn word(&self) -> &str {
        match self {
            &NameWordOrInitial::Word(word, _) => word,
            &NameWordOrInitial::Initial(_) => unreachable!(),
        }
    }

    fn has_word(&self) -> bool {
        match self {
            &NameWordOrInitial::Word(_, _) => true,
            &NameWordOrInitial::Initial(_) => false,
        }
    }

    fn initials_count(&self) -> u8 {
        match self {
            &NameWordOrInitial::Word(_, count) => count as u8,
            &NameWordOrInitial::Initial(_) => 1,
        }
    }
}
