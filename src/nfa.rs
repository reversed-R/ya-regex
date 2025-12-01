use std::collections::{HashMap, HashSet};

use crate::parser::Node;

struct Env {
    count: u32,
}

impl Env {
    fn new() -> Self {
        Self { count: 0 }
    }

    fn next(&mut self) -> NfaState {
        self.count += 1;

        NfaState(self.count)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct NfaState(u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum NfaTrans {
    Epsilon,
    Char(char),
}

#[derive(Debug, Clone)]
pub(crate) struct Nfa {
    start: NfaState,
    states: HashMap<NfaState, HashMap<NfaTrans, HashSet<NfaState>>>,
    accept: NfaState,
}

impl From<Node> for Nfa {
    fn from(value: Node) -> Self {
        let mut env = Env::new();

        Self::new(value, &mut env)
    }
}

impl Nfa {
    fn new(n: Node, env: &mut Env) -> Self {
        match n {
            Node::Char(c) => Self::new_char(c, env),
            Node::Concat(n1, n2) => Self::new_concat(*n1, *n2, env),
            Node::Or(n1, n2) => Self::new_or(*n1, *n2, env),
            Node::Repeat(n) => Self::new_repeat(*n, env),
        }
    }

    fn new_char(c: char, env: &mut Env) -> Self {
        let start = env.next();

        let mut states = HashMap::new();
        let mut start_trans = HashMap::new();
        let mut accepts = HashSet::new();
        let accept = env.next();
        accepts.insert(accept);
        start_trans.insert(NfaTrans::Char(c), accepts);
        states.insert(start, start_trans);

        Self {
            start,
            states,
            accept,
        }
    }

    fn new_concat(n1: Node, n2: Node, env: &mut Env) -> Self {
        let start = env.next();
        let accept = env.next();

        let mut states = HashMap::new();

        let nfa1 = Self::new(n1, env);
        let nfa2 = Self::new(n2, env);

        // start -- epsilon --> nfa1.start
        let mut start_trans = HashMap::new();
        let mut start_trans_accepts = HashSet::new();
        start_trans_accepts.insert(nfa1.start);
        start_trans.insert(NfaTrans::Epsilon, start_trans_accepts);

        states.insert(start, start_trans);

        // nfa1.accept -- epsilon --> nfa2.start
        let mut trans = HashMap::new();
        let mut trans_accepts = HashSet::new();
        trans_accepts.insert(nfa2.start);
        trans.insert(NfaTrans::Epsilon, trans_accepts);

        states.insert(nfa1.accept, trans);

        // nfa2.accept -- epsilon --> accept
        let mut nfa2_accept_trans = HashMap::new();
        let mut nfa2_accept_trans_accepts = HashSet::new();
        nfa2_accept_trans_accepts.insert(accept);
        nfa2_accept_trans.insert(NfaTrans::Epsilon, nfa2_accept_trans_accepts);

        states.insert(nfa2.accept, nfa2_accept_trans);

        states.extend(nfa1.states);
        states.extend(nfa2.states);

        Self {
            start,
            states,
            accept,
        }
    }

    fn new_or(n1: Node, n2: Node, env: &mut Env) -> Self {
        let start = env.next();
        let accept = env.next();

        let mut states = HashMap::new();

        let nfa1 = Self::new(n1, env);
        let nfa2 = Self::new(n2, env);

        // start -- epsilon --> _
        let mut start_trans = HashMap::new();
        let mut start_trans_accepts = HashSet::new();
        // start -- epsilon --> nfa1.start
        start_trans_accepts.insert(nfa1.start);
        // start -- epsilon --> nfa2.start
        start_trans_accepts.insert(nfa2.start);
        start_trans.insert(NfaTrans::Epsilon, start_trans_accepts);

        states.insert(start, start_trans);

        // nfa1.accept -- epsilon --> accept
        let mut a1_trans = HashMap::new();
        let mut a1_trans_accepts = HashSet::new();
        a1_trans_accepts.insert(accept);
        a1_trans.insert(NfaTrans::Epsilon, a1_trans_accepts);

        states.insert(nfa1.accept, a1_trans);

        // nfa2.accept -- epsilon --> accept
        let mut a2_trans = HashMap::new();
        let mut a2_trans_accepts = HashSet::new();
        a2_trans_accepts.insert(accept);
        a2_trans.insert(NfaTrans::Epsilon, a2_trans_accepts);

        states.insert(nfa2.accept, a2_trans);

        states.extend(nfa1.states);
        states.extend(nfa2.states);

        Self {
            start,
            states,
            accept,
        }
    }

    fn new_repeat(n: Node, env: &mut Env) -> Self {
        let start = env.next();
        let accept = env.next();

        let mut states = HashMap::new();

        let nfa = Self::new(n, env);

        // start -- epsilon --> _
        let mut start_trans = HashMap::new();
        let mut start_trans_accepts = HashSet::new();
        // start -- epsilon --> accept
        start_trans_accepts.insert(accept);
        // start -- epsilon --> nfa.start
        start_trans_accepts.insert(nfa.start);
        start_trans.insert(NfaTrans::Epsilon, start_trans_accepts);

        states.insert(start, start_trans);

        // nfa.accept -- epsilon --> _
        let mut a_trans = HashMap::new();
        let mut a_trans_accepts = HashSet::new();
        // nfa.accept -- epsilon --> start
        a_trans_accepts.insert(start);
        // nfa.accept -- epsilon --> accept
        a_trans_accepts.insert(accept);
        a_trans.insert(NfaTrans::Epsilon, a_trans_accepts);

        states.insert(nfa.accept, a_trans);

        states.extend(nfa.states);

        Self {
            start,
            states,
            accept,
        }
    }

    pub fn start(&self) -> NfaState {
        self.start
    }

    pub fn states(&self) -> &HashMap<NfaState, HashMap<NfaTrans, HashSet<NfaState>>> {
        &self.states
    }

    pub fn accept(&self) -> NfaState {
        self.accept
    }
}

impl NfaState {
    pub fn as_u32(&self) -> u32 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use crate::{nfa::Nfa, parser::Node};

    #[test]
    fn new_nfa() {
        // a(b|c)*

        let ast = Node::Concat(
            Box::new(Node::Char('a')),
            Box::new(Node::Repeat(Box::new(Node::Or(
                Box::new(Node::Char('b')),
                Box::new(Node::Char('c')),
            )))),
        );

        let nfa = Nfa::from(ast);

        panic!("{nfa:#?}");
    }
}
