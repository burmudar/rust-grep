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
        self.name == *other.to_string()
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
enum Matchers {
    Character(PredicateFn),
    Epsilon,
}

impl Eq for Matchers {}

impl PartialEq for Matchers {
    fn eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl std::hash::Hash for Matchers {
    fn hash<H: std::hash::Hasher>(&self, matcher: &mut H) {
        match self {
            // We use unique identifiers here 0, 1 for the predicates since we can't hash otherwise
            Matchers::Character(_) => 1.hash(matcher),
            Matchers::Epsilon => 0.hash(matcher),
        }
    }
}

impl Matchers {
    fn new_char(c: char) -> Matchers {
        Self::Character(Rc::new(move |other: char| c == other))
    }

    fn new_epsilon() -> Matchers {
        Self::Epsilon
    }

    fn matches(&self, input: &str, pos: usize) -> bool {
        if self.is_epsilon() {
            return true;
        }

        let predicate = match self {
            Self::Character(p) => p,
            Self::Epsilon => return true,
        };

        // if we don't have a character at this postion then just return false
        let c = if let Some(ch) = input.chars().nth(pos) {
            ch
        } else {
            return false;
        };
        predicate(c)
    }

    fn is_epsilon(&self) -> bool {
        matches!(self, Self::Epsilon)
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
    matcher: Matchers,
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
    pub fn add_transition(&mut self, to_state: State, matcher: Matchers) {
        self.transitions.push(Transition { to_state, matcher })
    }

    // unshift_transition puts the transition at the front meaning it's the highest priority
    pub fn unshift_transition(&mut self, to_state: State, matcher: Matchers) {
        self.transitions.insert(0, Transition { to_state, matcher })
    }
}

struct EngineState {
    pos: usize,
    state: String,
    memory: Vec<String>,
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
        self.states.len()
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

    fn add_states(&mut self, states: &[String]) {
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

    fn add_transition(&mut self, from: &str, to: &str, matcher: Matchers) {
        match self.states.take(&State::new(from)) {
            Some(mut s) => {
                print!("transition<{}, {}>:{} ", s.name, to, matcher.name());
                s.add_transition(State::new(to), matcher);
                println!("count: {}", s.transitions.len());
                self.states.insert(s);
            }
            None => panic!("'{}' state not found!", from),
        }
    }

    fn unshift_transition(&mut self, from: State, to: State, matcher: Matchers) {
        if let Some(mut s) = self.states.take(&from) {
            s.unshift_transition(to, matcher);
            self.states.insert(s);
        }
    }

    fn compute(&self, input: &str) -> bool {
        let mut stack = Vec::new();

        // Initial state
        stack.push(EngineState {
            pos: 0,
            state: self.initial_state.clone(),
            memory: Vec::new(),
        });

        while let Some(current) = stack.pop() {
            if self.is_ending_state(&current.state) {
                return true;
            }

            let transitions: &[Transition] = match self.get_state(&current.state) {
                Some(state) => &state.transitions,
                None => &[],
            };

            for idx in (0..transitions.len()).rev() {
                let t = &transitions[idx];
                if t.matcher.matches(input, current.pos) {
                    let copy_memory = if t.matcher.is_epsilon() {
                        // if we've been here before we continue the loop otherwise we'll get stuck
                        if current.memory.contains(&t.matcher.name().to_string()) {
                            continue;
                        }
                        // we haven't been here, so lets remember it
                        let mut copy = current.memory.clone();
                        copy.push(t.matcher.name().to_string());
                        copy
                    } else {
                        Vec::new()
                    };
                    let next_pos = if t.matcher.is_epsilon() {
                        current.pos
                    } else {
                        current.pos + 1
                    };
                    stack.push(EngineState {
                        pos: next_pos,
                        state: t.to_state.name.clone(),
                        memory: copy_memory,
                    });
                }
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use crate::dfa::NFAEngine;

    use super::Matchers;

    #[test]
    fn engine_construct_has_initial_state() {
        let engine = NFAEngine::new("hello");

        assert_eq!(engine.initial_state, "hello");
        assert_eq!(engine.state_len(), 1);
        assert!(engine.has_state("hello"));

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
            assert!(engine.has_state(state_name));
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
        assert!(engine.get_state("b").is_some());
    }

    #[test]
    fn compute() {
        let mut engine = NFAEngine::new_with_states("q0", &["q0", "q1", "q2", "q3"]);

        engine.set_ending_states(&["q3"]);
        engine.add_transition("q0", "q1", Matchers::new_char('a'));
        engine.add_transition("q1", "q2", Matchers::new_char('b'));
        engine.add_transition("q2", "q3", Matchers::new_epsilon());

        assert!(engine.compute("abbbbbb"));
        assert!(!engine.compute("aabbbbbb"));
        assert!(engine.compute("ab"));
        assert!(engine.compute("abc"));
        assert!(!engine.compute("a"));
    }

    #[test]
    fn stuck_forever() {
        let mut engine = NFAEngine::new_with_states("q0", &["q0", "q1", "q2"]);
        engine.set_ending_states(&["q2"]);
        engine.add_transition("q0", "q1", Matchers::new_char('a'));
        engine.add_transition("q1", "q1", Matchers::Epsilon);
        engine.add_transition("q1", "q2", Matchers::new_char('b'));

        assert!(engine.compute("ab"));
    }
}
