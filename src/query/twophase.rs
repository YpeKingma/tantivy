use std::cell::RefCell;
use std::rc::Rc;

/// Set of documents possibly matching a query within a specific segment.
pub trait TwoPhase: downcast_rs::Downcast + 'static {
    /// An estimate of the expected cost to determine that a single document `.matches()`.
    /// Returns an expected cost in number of simple operations like addition, multiplication,
    /// comparing two numbers and indexing an array.
    /// The returned value must be positive.
    fn match_cost(&self) -> f32;

    /// Return whether the current valid doc in the approximating DocSet is on a match.
    /// This should only be called when the DocSet is positioned, and at most once.
    /// The approximating DocSet implements the first phase, this method implements the second phase.
    fn matches(&mut self) -> bool;
}

impl TwoPhase for Rc<RefCell<dyn TwoPhase + 'static>> {
    fn match_cost(&self) -> f32 {
        let two_phase = &self.borrow();
        two_phase.match_cost()
    }

    fn matches(&mut self) -> bool {
        let two_phase = &mut self.borrow_mut();
        two_phase.matches()
    }
}
