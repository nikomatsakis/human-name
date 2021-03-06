use phf;
use std::ascii::AsciiExt;
use super::namepart::NamePart;

static TWO_CHAR_TITLES: [&'static str; 4] = ["mr", "ms", "sr", "dr"];

static PREFIX_TITLE_PARTS: phf::Set<&'static str> = phf_set! {
    "Aunt",
    "Auntie",
    "Attaché",
    "Dame",
    "Marchioness",
    "Marquess",
    "Marquis",
    "Marquise",
    "King",
    "King'S",
    "Queen",
    "Queen'S",
    "Abbess",
    "Abbot",
    "Academic",
    "Acolyte",
    "Adept",
    "Adjutant",
    "Adm",
    "Admiral",
    "Advocate",
    "Akhoond",
    "Air",
    "Ald",
    "Alderman",
    "Almoner",
    "Ambassador",
    "Amn",
    "Analytics",
    "Appellate",
    "Apprentice",
    "Arbitrator",
    "Archbishop",
    "Archdeacon",
    "Archdruid",
    "Archduchess",
    "Archduke",
    "Arhat",
    "Assistant",
    "Assoc",
    "Associate",
    "Asst",
    "Attache",
    "Attorney",
    "Ayatollah",
    "Baba",
    "Bailiff",
    "Banner",
    "Bard",
    "Baron",
    "Barrister",
    "Bearer",
    "Bench",
    "Bgen",
    "Bishop",
    "Blessed",
    "Bodhisattva",
    "Brig",
    "Brigadier",
    "Briggen",
    "Brother",
    "Buddha",
    "Burgess",
    "Business",
    "Bwana",
    "Canon",
    "Capt",
    "Captain",
    "Cardinal",
    "Chargé",
    "Catholicos",
    "Ccmsgt",
    "Cdr",
    "Ceo",
    "Cfo",
    "Chair",
    "Chairs",
    "Chancellor",
    "Chaplain",
    "Chief",
    "Chieftain",
    "Civil",
    "Clerk",
    "Cmd",
    "Cmdr",
    "Cmsaf",
    "Cmsgt",
    "Co-Chair",
    "Co-Chairs",
    "Coach",
    "Col",
    "Colonel",
    "Commander",
    "Commander-In-Chief",
    "Commodore",
    "Comptroller",
    "Controller",
    "Corporal",
    "Corporate",
    "Councillor",
    "Count",
    "Countess",
    "Courtier",
    "Cpl",
    "Cpo",
    "Cpt",
    "Credit",
    "Criminal",
    "Csm",
    "Curator",
    "Customs",
    "Cwo",
    "D'Affaires",
    "Deacon",
    "Delegate",
    "Deputy",
    "Designated",
    "Det",
    "Dir",
    "Director",
    "Discovery",
    "District",
    "Division",
    "Docent",
    "Docket",
    "Doctor",
    "Doyen",
    "Dpty",
    "Druid",
    "Duke",
    "Dutchess",
    "Edmi",
    "Edohen",
    "Effendi",
    "Ekegbian",
    "Elder",
    "Elerunwon",
    "Emperor",
    "Empress",
    "Ens",
    "Envoy",
    "Exec",
    "Executive",
    "Fadm",
    "Family",
    "Father",
    "Federal",
    "Field",
    "Financial",
    "First",
    "Flag",
    "Flying",
    "Flight",
    "Flt",
    "Foreign",
    "Forester",
    "Frau",
    "Friar",
    "Gen",
    "General",
    "Generalissimo",
    "Gentiluomo",
    "Giani",
    "Goodman",
    "Goodwife",
    "Governor",
    "Grand",
    "Group",
    "Guru",
    "Gyani",
    "Gysgt",
    "Hajji",
    "Headman",
    "Her",
    "Herr",
    "Hereditary",
    "High",
    "His",
    "Hon",
    "Honorable",
    "Honourable",
    "Imam",
    "Information",
    "Insp",
    "Intelligence",
    "Intendant",
    "Journeyman",
    "Judge",
    "Judicial",
    "Justice",
    "Junior",
    "Kingdom",
    "Knowledge",
    "Lady",
    "Lama",
    "Lamido",
    "Law",
    "Lcdr",
    "Lcpl",
    "Leader",
    "Lieutenant",
    "Lord",
    "Leut",
    "Lieut",
    "Ltc",
    "Ltcol",
    "Ltg",
    "Ltgen",
    "Ltjg",
    "Madam",
    "Madame",
    "Mag",
    "Mag-Judge",
    "Mag/Judge",
    "Magistrate",
    "Magistrate-Judge",
    "Maharajah",
    "Maharani",
    "Mahdi",
    "Maid",
    "Maj",
    "Majesty",
    "Majgen",
    "Major",
    "Manager",
    "Marcher",
    "Marketing",
    "Marshal",
    "Master",
    "Matriarch",
    "Matron",
    "Mayor",
    "Mcpo",
    "Mcpoc",
    "Mcpon",
    "Member",
    "Metropolitan",
    "Mgr",
    "Mgysgt",
    "Minister",
    "Miss",
    "Misses",
    "Mister",
    "Mme",
    "Monsignor",
    "Most",
    "Mother",
    "Mpco-Cg",
    "Mrs",
    "Msg",
    "Msgr",
    "Msgt",
    "Mufti",
    "Mullah",
    "Municipal",
    "Murshid",
    "Nanny",
    "National",
    "Nurse",
    "Officer",
    "Operating",
    "Pastor",
    "Patriarch",
    "Petty",
    "Pfc",
    "Pharaoh",
    "Pilot",
    "Pir",
    "Police",
    "Political",
    "Pope",
    "Prefect",
    "Prelate",
    "Premier",
    "Pres",
    "Presbyter",
    "President",
    "Presiding",
    "Priest",
    "Priestess",
    "Primate",
    "Prime",
    "Prin",
    "Prince",
    "Princess",
    "Principal",
    "Prior",
    "Private",
    "Pro",
    "Prof",
    "Professor",
    "Provost",
    "Pslc",
    "Pte",
    "Pursuivant",
    "Pvt",
    "Rabbi",
    "Radm",
    "Rangatira",
    "Ranger",
    "Rdml",
    "Rear",
    "Rebbe",
    "Registrar",
    "Rep",
    "Representative",
    "Resident",
    "Rev",
    "Revenue",
    "Reverend",
    "Reverand",
    "Revd",
    "Right",
    "Risk",
    "Royal",
    "Saint",
    "Sargent",
    "Sargeant",
    "Saoshyant",
    "Scpo",
    "Secretary",
    "Security",
    "Seigneur",
    "Senator",
    "Senior",
    "Senior-Judge",
    "Sergeant",
    "Servant",
    "Sfc",
    "Sgm",
    "Sgt",
    "Sgtmaj",
    "Sgtmajmc",
    "Shehu",
    "Sheikh",
    "Sheriff",
    "Siddha",
    "Sir",
    "Sister",
    "Sma",
    "Smsgt",
    "Solicitor",
    "Spc",
    "Speaker",
    "Special",
    "Sra",
    "Ssg",
    "Ssgt",
    "Staff",
    "State",
    "States",
    "Strategy",
    "Subaltern",
    "Subedar",
    "Sultan",
    "Sultana",
    "Superior",
    "Supreme",
    "Surgeon",
    "Swordbearer",
    "Sysselmann",
    "Tax",
    "Technical",
    "Timi",
    "Tirthankar",
    "Treasurer",
    "Tsar",
    "Tsarina",
    "Tsgt",
    "Uncle",
    "United",
    "Vadm",
    "Vardapet",
    "Venerable",
    "Verderer",
    "Very",
    "Vicar",
    "Vice",
    "Viscount",
    "Vizier",
    "Warden",
    "Warrant",
    "Wing",
    "Woodman",
    "And",
    "The",
    "Und",
};

static POSTFIX_TITLES: phf::Set<&'static str> = phf_set! {
    "Esq",
    "Esquire",
    "Attorney-at-law",
    "Et",
    "Al",
};

fn might_be_title_part(word: &NamePart) -> bool {
    if word.chars < 3 {
        // Allow any word with 1 or 2 characters as part of a title (but see below)
        true
    } else if !word.is_namelike() {
        true
    } else if word.word.chars().any(char::is_numeric) {
        true
    } else {
        PREFIX_TITLE_PARTS.contains(&*word.namecased)
    }
}

fn might_be_last_title_part(word: &NamePart) -> bool {
    // Don't allow 1 or 2-character words as the whole or final piece of
    // a title, except a set of very-common two-character title abbreviations,
    // because otherwise we are more likely dealing with initials
    if word.chars < 3 || word.word.chars().filter(|c| c.is_alphanumeric()).count() < 2 {
        word.chars == 2 && TWO_CHAR_TITLES.iter().any(|title| title.eq_ignore_ascii_case(word.word))
    } else {
        might_be_title_part(word)
    }
}

fn is_prefix_title(words: &[NamePart]) -> bool {
    match words.last() {
        Some(word) => {
            if !might_be_last_title_part(&word) {
                return false;
            }
        }
        None => {
            return false;
        }
    }

    if words.len() > 1 {
        words[0..words.len() - 1].iter().all(|word| might_be_title_part(&word))
    } else {
        true
    }
}

pub fn is_postfix_title(word: &NamePart, might_be_initials: bool) -> bool {
    if word.is_namelike() {
        POSTFIX_TITLES.contains(&*word.namecased) || word.word.chars().any(char::is_numeric)
    } else if word.is_initials() {
        !might_be_initials && word.word.chars().filter(|c| c.is_alphabetic()).count() > 1
    } else {
        true
    }
}

pub fn strip_prefix_title<'a>(words: &mut Vec<NamePart<'a>>) -> Option<Vec<NamePart<'a>>> {
    let mut prefix_len = words.len() - 1;
    while prefix_len > 0 {
        let found_prefix = {
            let next_word = &words[prefix_len];
            (next_word.is_namelike() || next_word.is_initials()) &&
            is_prefix_title(&words[0..prefix_len])
        };

        if found_prefix {
            return Some(words.drain(0..prefix_len).collect::<Vec<_>>());
        }

        prefix_len -= 1;
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::namepart::{Location, NamePart};

    #[test]
    fn is_postfix_title_esq() {
        let part = NamePart::from_word("esq", true, Location::Start);
        assert!(is_postfix_title(&part, true));
    }

    #[test]
    fn is_postfix_title_et_al() {
        let parts: Vec<_> = NamePart::all_from_text("et al", true, Location::Start).collect();
        for part in parts {
            assert!(is_postfix_title(&part, true));
        }
    }

    #[test]
    fn is_postfix_title_abbr() {
        let part = NamePart::from_word("asd.", true, Location::Start);
        assert!(is_postfix_title(&part, true));
    }

    #[test]
    fn is_postfix_title_initialism() {
        let part = NamePart::from_word("a.s.d.", true, Location::Start);
        assert!(is_postfix_title(&part, false));
        assert!(!is_postfix_title(&part, true));
    }

    #[test]
    fn strip_prefix_title_none() {
        let mut parts: Vec<_> = NamePart::all_from_text("Jane Doe", true, Location::Start)
                                    .collect();
        strip_prefix_title(&mut parts);
        assert_eq!("Jane Doe",
                   parts.iter().fold("".to_string(), |s, ref p| s + " " + p.word).trim());
    }

    #[test]
    fn strip_prefix_title_abbr() {
        let mut parts: Vec<_> = NamePart::all_from_text("Dr. Jane Doe", true, Location::Start)
                                    .collect();
        strip_prefix_title(&mut parts);
        assert_eq!("Jane Doe",
                   parts.iter().fold("".to_string(), |s, ref p| s + " " + p.word).trim());
    }

    #[test]
    fn strip_prefix_title_multi_abbr() {
        let mut parts: Vec<_> = NamePart::all_from_text("Revd. Dr. Jane Doe",
                                                        true,
                                                        Location::Start)
                                    .collect();
        strip_prefix_title(&mut parts);
        assert_eq!("Jane Doe",
                   parts.iter().fold("".to_string(), |s, ref p| s + " " + p.word).trim());
    }

    #[test]
    fn strip_prefix_title_word() {
        let mut parts: Vec<_> = NamePart::all_from_text("Lady Jane Doe", true, Location::Start)
                                    .collect();
        strip_prefix_title(&mut parts);
        assert_eq!("Jane Doe",
                   parts.iter().fold("".to_string(), |s, ref p| s + " " + p.word).trim());
    }

    #[test]
    fn strip_prefix_title_multi_word() {
        let mut parts: Vec<_> = NamePart::all_from_text("1st (B) Ltc Jane Doe",
                                                        true,
                                                        Location::Start)
                                    .collect();
        strip_prefix_title(&mut parts);
        assert_eq!("Jane Doe",
                   parts.iter().fold("".to_string(), |s, ref p| s + " " + p.word).trim());
    }

    #[test]
    fn strip_prefix_title_short() {
        let mut parts: Vec<_> = NamePart::all_from_text("Dr. Doe", true, Location::Start).collect();
        strip_prefix_title(&mut parts);
        assert_eq!("Doe",
                   parts.iter().fold("".to_string(), |s, ref p| s + " " + p.word).trim());
    }
}
