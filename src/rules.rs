pub enum Decision {
    Allow,
    Deny,
}

#[derive(Debug, Eq, PartialEq, Default, serde::Deserialize)]
pub struct RuleSet {
    #[serde(default)]
    pub allow: Vec<CharacterType>,
    #[serde(default)]
    pub deny: Vec<CharacterType>,
}

impl RuleSet {
    pub fn decision(&self, c: char) -> Option<Decision> {
        let allow_specificity = self
            .allow
            .iter()
            .filter(|rule| rule.matches(c))
            .map(|rule| rule.specificity())
            .max();
        let deny_specificity = self
            .deny
            .iter()
            .filter(|rule| rule.matches(c))
            .map(|rule| rule.specificity())
            .max();
        match (allow_specificity, deny_specificity) {
            (Some(_), None) => Some(Decision::Allow),
            (None, Some(_)) => Some(Decision::Deny),
            (None, None) => None,
            (Some(allow_specificity), Some(deny_specificity)) => {
                if deny_specificity >= allow_specificity {
                    Some(Decision::Deny)
                } else {
                    Some(Decision::Allow)
                }
            }
        }
    }
}

#[derive(Debug)]
pub enum CharacterType {
    /// Single character (eg. "U+9000")
    CodePoint(char),
    /// An inclusive range of characters (eg. "U+1400..U+1409")
    Range(unic_char_range::CharRange),
    /// All bidirectional control characters (right to left etc)
    Bidi,
    /// Named ranges of characters (eg. "Tibetan", "Box Drawing")
    Block(unic_ucd_block::Block),
    /// Any possible character.
    Anything,
}

impl CharacterType {
    fn matches(&self, c: char) -> bool {
        match self {
            Self::CodePoint(rule_char) => *rule_char == c,
            Self::Range(range) => range.contains(c),
            Self::Bidi => [
                '\u{202A}', '\u{202b}', '\u{202c}', '\u{202d}', '\u{202e}', '\u{2066}', '\u{2067}',
                '\u{2068}', '\u{2069}',
            ]
            .contains(&c),
            Self::Block(block) => block.range.contains(c),
            Self::Anything => true,
        }
    }

    fn specificity(&self) -> u32 {
        match self {
            Self::CodePoint(..) => 5,
            Self::Range(_) => 4,
            Self::Bidi => 3,
            Self::Block(..) => 2,
            Self::Anything => 1,
        }
    }
}

impl PartialEq for CharacterType {
    fn eq(&self, other: &Self) -> bool {
        use CharacterType::*;
        match (self, other) {
            (CodePoint(self_c), CodePoint(other_c)) => self_c == other_c,
            (Range(self_r), Range(other_r)) => self_r == other_r,
            (Bidi, Bidi) => true,
            (Block(self_block), Block(other_block)) => self_block.name == other_block.name,
            (Anything, Anything) => true,
            _ => false,
        }
    }
}

impl Eq for CharacterType {}
