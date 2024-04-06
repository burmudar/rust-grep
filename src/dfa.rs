use std::collections::HashSet;
use std::rc::Rc;

#[derive(Eq, Clone)]
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

impl std::hash::Hash for State {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state)
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

    fn matches(&self, c: char) -> bool {
        let predicate = match self {
            Self::Character(p) => p,
            Self::Epsilon => return true,
        };

        predicate(c)
    }

    fn is_epsilon(&self) -> bool {
        match self {
            Self::Epsilon => true,
            _ => false,
        }
    }

    fn name(&self) -> &str {
        match self {
            Self::Character(_) => "Character",
            Self::Epsilon => "Epsilon",
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

struct EngineState {
    pos: usize,
    state: String,
}

struct NFAEngine {
    states: HashSet<State>,
    initial_state: String,
    ending_states: Vec<String>,
}

impl NFAEngine {
    fn new_with_states(initial: &str, states: &[&str]) -> NFAEngine {
        let mut engine = NFAEngine::new(initial);
        engine.declare_states_with_names(states);
        engine
    }

    fn new(initial: &str) -> NFAEngine {
        let mut states = HashSet::new();
        states.insert(State::new(initial));
        NFAEngine {
            states,
            initial_state: initial.to_string(),
            ending_states: Vec::new(),
        }
    }

    fn state_len(&self) -> usize {
        return self.states.len();
    }

    fn has_state(&self, state: &str) -> bool {
        self.states.contains(&State::new(state))
    }

    fn get_state(&self, name: &str) -> Option<&State> {
        self.states.get(&State::new(name))
    }

    fn add_state(&mut self, state: &str) -> bool {
        self.states.insert(State::new(state))
    }

    fn add_states(&mut self, states: &Vec<String>) {
        states.iter().for_each(|s| {
            self.add_state(s);
        })
    }

    fn declare_states_with_names(&mut self, names: &[&str]) {
        for &n in names {
            self.add_state(n);
        }
    }

    fn set_initial_state(&mut self, state: &str) {
        if self.has_state(state) {
            self.initial_state = state.to_string()
        } else {
            panic!("state '{}' does not exist", state)
        }
    }

    fn set_ending_states(&mut self, states: &[&str]) {
        states.iter().for_each(|s| {
            if !self.has_state(s) {
                self.add_state(s);
            }
            self.ending_states.clear();
            states
                .iter()
                .for_each(|s| self.ending_states.push(s.to_string()));
        })
    }

    fn is_ending_state(&self, state: &str) -> bool {
        self.ending_states.contains(&state.to_string())
    }

    fn add_transition(&mut self, from: &str, to: &str, matcher: Matcher) {
        match self.states.take(&State::new(from)) {
            Some(mut s) => {
                print!("transition<{}, {}>:{}", s.name, to, matcher.name());
                s.add_transition(State::new(to), matcher);
                println!("count: {}", s.transitions.len());
                self.states.insert(s);
            }
            None => panic!("'{}' state not found!", from),
        }
    }

    fn unshift_transition(&mut self, from: State, to: State, matcher: Matcher) {
        if let Some(mut s) = self.states.take(&from) {
            s.unshift_transition(to, matcher);
            self.states.insert(s);
        }
    }

    fn compute(&self, value: &str) -> bool {
        let mut stack = Vec::new();

        stack.push(EngineState {
            pos: 0,
            state: self.initial_state.clone(),
        });

        while let Some(current) = stack.pop() {
            println!("current state: {}", current.state);
            if self.is_ending_state(&current.state) {
                return true;
            }

            let transitions: &[Transition] = match self.get_state(&current.state) {
                Some(state) => &state.transitions,
                None => &[],
            };
            transitions.iter().rev().for_each(|t: &Transition| {
                match value.chars().nth(current.pos) {
                    Some(c) => {
                        println!(
                            "transition\n c: {}\n to: {}\n matcher:{}",
                            c,
                            t.to_state.name,
                            t.matcher.name()
                        );

                        if t.matcher.matches(c) {
                            let next_pos = match t.matcher.is_epsilon() {
                                true => current.pos,
                                false => current.pos + 1,
                            };
                            stack.push(EngineState {
                                pos: next_pos,
                                state: t.to_state.name.clone(),
                            })
                        }
                    }
                    _ => (),
                };
            })
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use crate::dfa::NFAEngine;

    use super::{Matcher, State};

    #[test]
    fn engine_construct_has_initial_state() {
        let engine = NFAEngine::new("hello");

        assert_eq!(engine.initial_state, "hello");
        assert_eq!(engine.state_len(), 1);
        assert_eq!(engine.has_state("hello"), true);

        engine
            .get_state("hello")
            .expect("'hello' state should exist");
    }

    #[test]
    fn engine_has_declared_states() {
        let mut engine = NFAEngine::new("a");

        let extra_states = ["a", "b", "c"];
        engine.declare_states_with_names(&extra_states);

        // since our initial state is 'a' and one of our extra states 'a' matches, we only expect a
        // total of 3 final states
        assert_eq!(engine.state_len(), extra_states.len());

        for state_name in extra_states {
            assert_eq!(engine.has_state(state_name), true);
        }
    }

    #[test]
    fn engine_has_ending_states() {
        let mut engine = NFAEngine::new("a");

        let ending_states = &["a", "b"];

        assert_eq!(engine.state_len(), 1);
        // set ending states  should add a state if it doesn't exist
        engine.set_ending_states(ending_states);
        // "b" should have been added so now we have 2 states
        assert_eq!(engine.state_len(), 2);

        // let's get "b"
        assert!(matches!(engine.get_state("b"), Some(_)));
    }

    #[test]
    fn compute() {
        let mut engine = NFAEngine::new_with_states("q0", &["q0", "q1", "q2", "q3"]);

        engine.set_ending_states(&["q3"]);
        engine.add_transition("q0", "q1", Matcher::new_char('a'));
        engine.add_transition("q1", "q2", Matcher::new_char('b'));
        engine.add_transition("q2", "q2", Matcher::new_char('b'));
        engine.add_transition("q2", "q3", Matcher::new_epsilon());

        // assert_eq!(engine.compute("abbbbbb"), true);
        // assert_eq!(engine.compute("aabbbbbb"), false);
        assert_eq!(engine.compute("ab"), true);
        assert_eq!(engine.compute("a"), false);
    }
}
