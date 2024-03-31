use std::collections::HashSet;

#[derive(Eq, PartialEq, Hash)]
pub struct State {
    name: String,
    transitions: Vec<Transition>,
    start_groups: Vec<String>,
    end_groups: Vec<String>,
}

type PredicateFn = Box<dyn Fn(char) -> bool>;

enum Matcher {
    Character(PredicateFn),
    Epsilon,
}

impl Eq for Matcher {}

impl PartialEq for Matcher {
    fn eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl std::hash::Hash for Matcher {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.hash(state)
    }
}

impl Matcher {
    fn new_char(c: char) -> Matcher {
        Self::Character(Box::new(move |other: char| c == other))
    }

    fn new_epsilon() -> Matcher {
        Self::Epsilon
    }

    fn matches(self: Self, c: char) -> bool {
        let predicate = match self {
            Self::Character(p) => p,
            Self::Epsilon => return true,
        };

        predicate(c)
    }

    fn is_epsilon(self: Self) -> bool {
        match self {
            Self::Epsilon => true,
            _ => false,
        }
    }
}

#[derive(Eq, PartialEq, Hash)]
struct Transition {
    to_state: State,
    matcher: Matcher,
}

impl State {
    pub fn new(name: String) -> State {
        State {
            name,
            transitions: Vec::new(),
            start_groups: Vec::new(),
            end_groups: Vec::new(),
        }
    }

    // add_transition adds the transition to the end of the list of transitions
    pub fn add_transition(&mut self, to_state: State, matcher: Matcher) {
        self.transitions.push(Transition { to_state, matcher })
    }

    // unshift_transition puts the transition at the front meaning it's the highest priority
    pub fn unshift_transition(&mut self: State, to_state: State, matcher: Matcher) {
        self.transitions.insert(0, Transition { to_state, matcher })
    }
}

struct NFAEngine {
    states: HashSet<State>,
    initial_state: State,
    ending_states: HashSet<State>,
}

impl NFAEngine {
    fn new(initial: State) -> NFAEngine {
        NFAEngine {
            states: HashSet::new(),
            initial_state: initial,
            ending_states: HashSet::new(),
        }
    }

    fn set_initial_state(self: &mut Self, state: State) {
        self.initial_state = state
    }

    fn add_transition(&mut self, from: State, to: State, matcher: Matcher) {
        if let Some(mut s) = self.states.take(&from) {
            s.add_transition(to, matcher);
            self.states.insert(s);
        }
    }
}
