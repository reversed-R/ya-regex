use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};

use crate::nfa::{Nfa, NfaState, NfaTrans};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct DfaState(u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct DfaTrans(char);

#[derive(Debug, Clone)]
pub(crate) struct Dfa {
    start: DfaState,
    states: HashMap<DfaState, HashMap<DfaTrans, DfaState>>,
    accepts: HashSet<DfaState>,
}

struct Env {
    state_map: HashMap<NfaStateSet, (HashMap<DfaTrans, NfaStateSet>, DfaState)>,
}

impl Env {
    fn new() -> Self {
        Self {
            state_map: HashMap::new(),
        }
    }

    fn insert(&mut self, nstat: NfaStateSet, transs: HashMap<DfaTrans, NfaStateSet>) -> bool {
        if let Some((ts, s)) = self.state_map.get_mut(&nstat) {
            ts.extend(transs);

            false
        } else {
            let s = DfaState(self.state_map.len() as u32);

            self.state_map.insert(nstat, (transs, s));

            true
        }
    }

    fn into_dfa_states(self) -> HashMap<DfaState, HashMap<DfaTrans, DfaState>> {
        let mut states = HashMap::new();

        for (ns, (map, _)) in self.state_map.iter() {
            let (_, ds) = self.state_map.get(ns).unwrap();

            states.insert(
                ds.to_owned(),
                map.iter()
                    .map(|(t, ns)| (t.to_owned(), self.state_map.get(ns).unwrap().1))
                    .collect(),
            );
        }

        states
    }
}

impl From<Nfa> for Dfa {
    fn from(value: Nfa) -> Self {
        let mut env = Env::new();

        let start = NfaStateSet(value.next(&value.start(), &NfaTrans::Epsilon));
        let mut nstats = HashSet::new();
        nstats.insert(start.clone());

        let mut new = nstats.clone();

        while !new.is_empty() {
            let mut nexts = HashSet::<NfaStateSet>::new();

            for ns in new.into_iter() {
                let trans_map = value.transition_map_of(&ns);

                nexts.extend(
                    trans_map
                        .values()
                        .map(|ns| ns.clone())
                        .collect::<HashSet<_>>(),
                );

                env.insert(ns, trans_map);
            }

            new = nexts.difference(&nstats).map(|nss| nss.clone()).collect();
            nstats.extend(new.clone());
        }

        let (_, dstart) = env.state_map.get(&start).unwrap();

        Self {
            start: *dstart,
            states: env.into_dfa_states(),
            accepts: HashSet::new(),
        }
    }
}

// NfaStateSet はNfaの状態の集合だが、ただし、この集合に含まれる状態からのepsilon遷移は必ず全てこの集合にに帰着する、閉じた集合であるものとする
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NfaStateSet(HashSet<NfaState>);

impl Hash for NfaStateSet {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let mut mul = 1;

        for s in self.0.iter() {
            mul *= s.as_u32();
        }

        mul.hash(state);
    }
}

impl Nfa {
    fn transition_map_of(&self, states: &NfaStateSet) -> HashMap<DfaTrans, NfaStateSet> {
        let mut nexts = HashMap::<DfaTrans, NfaStateSet>::new();

        for s in states.0.iter() {
            if let Some(transs) = self.states().get(s) {
                for (t, stats) in transs.iter() {
                    if let &NfaTrans::Char(c) = t {
                        if let Some(nxts) = nexts.get_mut(&DfaTrans(c)) {
                            nxts.0.extend(stats);
                        } else {
                            nexts.insert(DfaTrans(c), NfaStateSet(stats.clone()));
                        }
                    }
                    // epsilon遷移は集合自身に戻ることが保証されているため、考慮しなくて良い
                }
            }
        }

        nexts
    }

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

struct Regex {
    nfa: Nfa,
}

impl Regex {
    fn matches(&self, pattern: &str) -> bool {
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
    use crate::{
        dfa::{Dfa, Regex},
        nfa::Nfa,
        parser::Node,
    };

    #[test]
    fn dfa_from_nfa() {
        // a(b|c)*

        let ast = Node::Concat(
            Box::new(Node::Char('a')),
            Box::new(Node::Repeat(Box::new(Node::Or(
                Box::new(Node::Char('b')),
                Box::new(Node::Char('c')),
            )))),
        );

        let nfa = Nfa::from(ast);

        let dfa = Dfa::from(nfa);

        panic!("{dfa:#?}");
    }

    #[test]
    fn regex_works() {
        // a(b|c)*

        let ast = Node::Concat(
            Box::new(Node::Char('a')),
            Box::new(Node::Repeat(Box::new(Node::Or(
                Box::new(Node::Char('b')),
                Box::new(Node::Char('c')),
            )))),
        );

        let nfa = Nfa::from(ast);

        let regex = Regex { nfa };

        assert!(regex.matches("a"));
        assert!(regex.matches("ab"));
        assert!(regex.matches("ac"));
        assert!(!regex.matches("b"));
        assert!(!regex.matches("bcb"));
        assert!(regex.matches("acbbc"));
    }
}
