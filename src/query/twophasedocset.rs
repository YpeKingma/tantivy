use crate::docset::{DocSet, SkipResult};
use crate::DocId;

struct TwoPhaseApproximation<TDocSet: DocSet> {
   approximation: TDocSet,
}

impl<TDocSet: DocSet> TwoPhaseApproximation<TDocSet> {
    pub fn new(approximation: TDocSet) -> dyn TwoPhaseDocSet<TDocSet> {
        TwoPhaseDoc::<TDocSet> {
            approximation
        }
    }

    pub fn approximation(self) -> TDocSet {
        self.approximation
    }
}

pub trait TwoPhaseDocSet {
    // An estimate of the expected cost to determine that a single document `.matches()`.
    // Returns an expected cost in number of simple operations like addition, multiplication,
    // comparing two numbers and indexing an array.
    // The returned value must be positive.
    fn match_cost() -> f32;

    // Return whether the current valid doc in the approximating DocSet is on a match.
    // This should only be called when the DocSet is positioned, and at most once.
    // The approximating DocSet implements the first phase, this method implements the second phase.
    fn matches(&mut self) -> bool;
}

impl<TDocSet: DocSet> DocSet for TwoPhaseApproximation<TDocSet> {
    // much like ConstScorer in scorer
    fn advance(&mut self) -> bool {
        self.approximation.advance()
    }

    fn skip_next(&mut self, target: DocId) -> SkipResult {
        self.approximation.skip_next(target)
    }

    fn doc(&self) -> DocId {
        self.approximation.doc()
    }

    fn size_hint(&self) -> u32 {
        self.approximation.size_hint()
    }

}
