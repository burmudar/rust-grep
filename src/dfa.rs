use std::collections::HashSet;
use std::rc::Rc;

#[derive(Eq, Clone, Hash)]
pub struct State {
    name: String,
    transitions: Vec<Transition>,
    start_groups: Vec<String>,
    end_groups: Vec<String>,
}

impl PartialEq<State> for State {
    fn eq(&self, other: &State) -> bool {
        // This might bite us in the foot later but we only want to consider similarly named
        // states!
        self.name == other.name
    }
}

// This allows us to do something like:
// let s = State { name: "william" };
// s == "william"
impl PartialEq<&str> for State {
    fn eq(&self, other: &&str) -> bool {
        self.name == other.to_string()
    }
}

// We use  Rc instead of Box, since the predicate can be safely shared so we just have to do a
// reference count
type PredicateFn = Rc<dyn Fn(char) -> bool>;

// We can clone matcher because it is safe to share PredicateFn since it doesn't modify anything
#[derive(Clone)]
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
    fn hash<H: std::hash::Hasher>(&self, matcher: &mut H) {
        match self {
            // We use unique identifiers here 0, 1 for the predicates since we can't hash otherwise
            Matcher::Character(_) => 1.hash(matcher),
            Matcher::Epsilon => 0.hash(matcher),
        }
    }
}

impl Matcher {
    fn new_char(c: char) -> Matcher {
        Self::Character(Rc::new(move |other: char| c == other))
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

#[derive(Eq, PartialEq, Hash, Clone)]
struct Transition {
    to_state: State,
    matcher: Matcher,
}

impl State {
    pub fn new(name: &str) -> State {
        State {
            name: name.to_string(),
            transitions: Vec::new(),
            start_groups: Vec::new(),
            end_groups: Vec::new(),
        }
    }

    pub fn from_collection(names: &[&str]) -> Vec<State> {
        names.iter().map(|&name| State::new(name)).collect()
    }

    // add_transition adds the transition to the end of the list of transitions
    pub fn add_transition(&mut self, to_state: State, matcher: Matcher) {
        self.transitions.push(Transition { to_state, matcher })
    }

    // unshift_transition puts the transition at the front meaning it's the highest priority
    pub fn unshift_transition(&mut self, to_state: State, matcher: Matcher) {
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
        let mut states = HashSet::new();
        states.insert(initial.clone());
        NFAEngine {
            states,
            initial_state: initial,
            ending_states: HashSet::new(),
        }
    }

    fn from(name: &str) -> NFAEngine {
        NFAEngine::new(State::new(name))
    }

    fn state_len(&self) -> usize {
        return self.states.len();
    }

    fn has_state(&self, state: &State) -> bool {
        self.states.contains(state)
    }

    fn get_state(&self, name: &str) -> Option<&State> {
        self.states.get(&State::new(name))
    }

    fn has_state_name(&self, name: &str) -> bool {
        self.has_state(&State::new(name))
    }

    fn add_state(&mut self, state: &State) -> bool {
        self.states.insert(state.clone())
    }

    fn add_states(&mut self, states: Vec<State>) {
        states.iter().for_each(|s| {
            self.add_state(s);
        })
    }

    fn declare_states_with_names(&mut self, names: &[&str]) {
        for &n in names {
            self.add_state(&State::new(n));
        }
    }

    fn set_initial_state(&mut self, state: State) {
        self.initial_state = state
    }

    fn set_ending_states(&mut self, states: Vec<State>) {
        states.iter().for_each(|s| {
            if !self.has_state(s) {
                self.add_state(s);
            }
            self.ending_states.insert(s.clone());
        })
    }

    fn add_transition(&mut self, from: State, to: State, matcher: Matcher) {
        if let Some(mut s) = self.states.take(&from) {
            s.add_transition(to, matcher);
            self.states.insert(s);
        }
    }

    fn unshift_transition(&mut self, from: State, to: State, matcher: Matcher) {
        if let Some(mut s) = self.states.take(&from) {
            s.unshift_transition(to, matcher);
            self.states.insert(s);
        }
    }

    fn compute(self, value: String) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use crate::dfa::NFAEngine;

    use super::State;

    #[test]
    fn engine_construct_has_initial_state() {
        let engine = NFAEngine::from("hello");

        assert_eq!(engine.initial_state.name, "hello");
        assert_eq!(engine.state_len(), 1);
        assert_eq!(engine.has_state_name("hello"), true);

        let state = engine
            .get_state("hello")
            .expect("'hello' state should exist");
        assert_eq!(engine.has_state(state), true)
    }

    #[test]
    fn engine_has_declared_states() {
        let mut engine = NFAEngine::from("a");

        let extra_states = ["a", "b", "c"];
        engine.declare_states_with_names(&extra_states);

        // since our initial state is 'a' and one of our extra states 'a' matches, we only expect a
        // total of 3 final states
        assert_eq!(engine.state_len(), extra_states.len());

        for state_name in extra_states {
            assert_eq!(engine.has_state_name(state_name), true);
        }
    }

    #[test]
    fn engine_has_ending_states() {
        let mut engine = NFAEngine::from("a");

        let ending_states = State::from_collection(&["a", "b"]);

        assert_eq!(engine.state_len(), 1);
        // set ending states  should add a state if it doesn't exist
        engine.set_ending_states(ending_states);
        // "b" should have been added so now we have 2 states
        assert_eq!(engine.state_len(), 2);

        // let's get "b"
        assert!(matches!(engine.get_state("b"), Some(_)));
    }
}
