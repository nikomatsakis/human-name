use phf;
use super::utils::capitalize_and_normalize;

// Store capitalized versions because we check after doing the initial,
// naive capitalization

static UNCAPITALIZED_PARTICLES: phf::Set<&'static str> = phf_set! {
    "Af",
    "Av",
    "Da",
    "Das",
    "Dal",
    "De",
    "Del",
    "Dela",
    "Dei",
    "Der",
    "Di",
    "Dí",
    "Do",
    "Dos",
    "Du",
    "La",
    "Le",
    "Na",
    "Ter",
    "Van",
    "Vel",
    "Von",
    "Zu",
    "Zum",
    "E",
    "Y",
};

static MAC_EXCEPTIONS: phf::Set<&'static str> = phf_set! {
    "Machin",
    "Machlin",
    "Machar",
    "Mackle",
    "Macklin",
    "Mackie",
    "Macevicius",
    "Maciulis",
    "Macias",
};

fn capitalize_after_mac(word: &str) -> bool {
    if word.len() <= 4 {
        false
    } else if word.ends_with('o') && word != "Macmurdo" {
        false
    } else if ["a", "c", "i", "z", "j"].iter().any(|c| word.ends_with(c)) {
        false
    } else if MAC_EXCEPTIONS.contains(word) {
        false
    } else {
        true
    }
}

pub fn namecase(word: &str, might_be_particle: bool) -> String {
    let result = capitalize_and_normalize(word);

    if might_be_particle && UNCAPITALIZED_PARTICLES.contains(&*result) {
        result.to_lowercase()
    } else if result.starts_with("Mac") && capitalize_after_mac(&result) {
        "Mac".to_string() + &capitalize_and_normalize(&result[3..])
    } else if result.starts_with("Mc") && result.len() > 3 {
        "Mc".to_string() + &capitalize_and_normalize(&result[2..])
    } else if result.starts_with("Al-") && result.len() > 3 {
        "al-".to_string() + &result[3..]
    } else {
        // Normal case
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple() {
        assert_eq!("Doe", namecase("doe", true));
    }

    #[test]
    fn conjunction() {
        assert_eq!("y", namecase("y", true));
        assert_eq!("Y", namecase("y", false));
    }

    #[test]
    fn particle() {
        assert_eq!("de", namecase("de", true));
        assert_eq!("De", namecase("de", false));
    }

    #[test]
    fn mcallen() {
        assert_eq!("McAllen", namecase("mcallen", true));
    }

    #[test]
    fn macmurdo() {
        assert_eq!("MacMurdo", namecase("macmurdo", true));
    }

    #[test]
    fn machlin() {
        assert_eq!("Machlin", namecase("machlin", true));
    }

    #[test]
    fn maciej() {
        assert_eq!("Maciej", namecase("maciej", true));
    }

    #[test]
    fn mach() {
        assert_eq!("Mach", namecase("mach", true));
    }

    #[test]
    fn macadaidh() {
        assert_eq!("MacAdaidh", namecase("macadaidh", true));
    }

    #[test]
    fn al_amir() {
        assert_eq!("al-Amir", namecase("al-amir", true));
    }
}
