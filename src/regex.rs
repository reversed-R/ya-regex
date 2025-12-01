use std::collections::HashSet;

use crate::{
    lexer,
    nfa::{Nfa, NfaState, NfaTrans},
    parser::{Node, ParseError},
};

impl Nfa {
    fn states_next(&self, states: &HashSet<NfaState>, trans: &NfaTrans) -> HashSet<NfaState> {
        let mut nexts = HashSet::new();

        for s in states.iter() {
            nexts.extend(self.next(s, trans));
        }

        nexts
    }

    fn next(&self, state: &NfaState, trans: &NfaTrans) -> HashSet<NfaState> {
        // 現在の状態 state から epsilon遷移で到達可能な状態の集合 e_starts を取得
        let mut starts = HashSet::new();
        starts.insert(state.to_owned());
        let e_starts = self.epsilon_next(starts);

        // 指定された遷移がepsilon遷移の場合、e_starts が求める状態の集合であるため直ちに終了
        if trans == &NfaTrans::Epsilon {
            e_starts
        } else {
            // 指定された遷移がepsilon遷移でない場合、e_starts からそのように遷移した集合 nexts を取得
            let mut nexts = HashSet::new();
            for s in e_starts.iter() {
                if let Some(transs) = self.states().get(s)
                    && let Some(t_nexts) = transs.get(trans)
                {
                    nexts.extend(t_nexts);
                }
            }

            // nexts からepsilon遷移して得られる集合が求める集合
            self.epsilon_next(nexts)
        }
    }

    fn epsilon_next(&self, states: HashSet<NfaState>) -> HashSet<NfaState> {
        let mut nexts = states.clone();
        let mut new = states;

        while !new.is_empty() {
            let next_new = self.transit_epsilon(&new);
            new = next_new.difference(&nexts).map(|n| n.to_owned()).collect();
            nexts.extend(next_new);
        }

        nexts
    }

    fn transit_epsilon(&self, states: &HashSet<NfaState>) -> HashSet<NfaState> {
        let mut nexts = HashSet::new();

        for s in states.iter() {
            if let Some(transs) = self.states().get(s)
                && let Some(epsilon_nexts) = transs.get(&NfaTrans::Epsilon)
            {
                nexts.extend(epsilon_nexts);
            }
        }

        nexts
    }
}

#[derive(Debug, Clone)]
pub struct Regex {
    nfa: Nfa,
}

#[derive(Debug, Clone, Copy)]
pub enum RegexParseError {
    UnexpectedEOF,
    UnexpectedToken,
}

impl From<ParseError> for RegexParseError {
    fn from(value: ParseError) -> Self {
        match value {
            ParseError::UnexpectedEOF => Self::UnexpectedEOF,
            ParseError::UnexpectedToken => Self::UnexpectedToken,
        }
    }
}

impl Regex {
    pub fn new(re: &str) -> Result<Self, RegexParseError> {
        let tokens = lexer::tokenize(re);
        let ast = Node::parse(&tokens).map_err(RegexParseError::from)?;
        let nfa = Nfa::from(ast);

        Ok(Self { nfa })
    }

    pub fn matches(&self, pattern: &str) -> bool {
        let mut states = HashSet::new();
        states.insert(self.nfa.start());

        for c in pattern.chars() {
            states = self.nfa.states_next(&states, &NfaTrans::Char(c));

            if states.is_empty() {
                return false;
            }
        }

        states.contains(&self.nfa.accept())
    }
}

#[cfg(test)]
mod tests {
    use crate::regex::Regex;

    #[test]
    fn regex_works() {
        let regex = Regex::new("a(b|c)*").unwrap();

        assert!(regex.matches("a"));
        assert!(regex.matches("ab"));
        assert!(regex.matches("ac"));
        assert!(!regex.matches("b"));
        assert!(!regex.matches("bcb"));
        assert!(regex.matches("acbbc"));
    }

    #[test]
    fn regex_works2() {
        let regex = Regex::new("a*b*").unwrap();

        assert!(regex.matches("")); // 空文字可
        assert!(regex.matches("a"));
        assert!(regex.matches("aa"));
        assert!(regex.matches("b"));
        assert!(regex.matches("bb"));
        assert!(regex.matches("aaabbb"));
        assert!(regex.matches("aaa"));
        assert!(regex.matches("bbb"));
        assert!(!regex.matches("abba")); // a* の後は b* のみ
        assert!(!regex.matches("ba")); // b の後に a は不可
    }

    #[test]
    fn regex_works3() {
        let regex = Regex::new("(ab)*|c").unwrap();

        assert!(regex.matches("")); // (ab)* → 空文字 ok
        assert!(regex.matches("ab"));
        assert!(regex.matches("abab"));
        assert!(regex.matches("ababab"));
        assert!(regex.matches("c"));
        assert!(!regex.matches("a")); // ab の途中
        assert!(!regex.matches("b"));
        assert!(!regex.matches("abc")); // 全体一致ではない
        assert!(!regex.matches("cab")); // 全体一致ではない
        assert!(!regex.matches("cc")); // c 1文字のみ
    }

    #[test]
    fn regex_works4() {
        let regex = Regex::new("a(bc)*d").unwrap();

        assert!(regex.matches("ad")); // bc が0回
        assert!(regex.matches("abcd")); // bc 1回
        assert!(regex.matches("abcbcd")); // bc 2回
        assert!(regex.matches("abcbcbcd")); // bc 3回
        assert!(!regex.matches("a")); // d が無い
        assert!(!regex.matches("d")); // 先頭が a でない
        assert!(!regex.matches("abc")); // 末尾が d でない
        assert!(!regex.matches("abcbd")); // bc の途中壊れ
        assert!(!regex.matches("aabcbcd")); // 最初に a が2つ
        assert!(!regex.matches("abcbcccd")); // bc の途中に c が余分
    }

    // #[test]
    // fn regex_works2() {
    //
    // }
}
