use crate::docset::{DocSet, SkipResult};
use crate::common::BitSet;
use crate::DocId;

struct TwoPhaseApproximation<TDocSet: DocSet> {
    approximation: TDocSet,
}

impl<TDocSet: DocSet> TwoPhaseApproximation<TDocSet> {
    pub fn new(approximation: TDocSet) -> TwoPhaseApproximation<TDocSet> {
        TwoPhaseApproximation::<TDocSet> { approximation }
    }

    pub fn approximation(self) -> TDocSet {
        self.approximation
    }
}

pub trait TwoPhaseDocSet: DocSet {
    // An estimate of the expected cost to determine that a single document `.matches()`.
    // Returns an expected cost in number of simple operations like addition, multiplication,
    // comparing two numbers and indexing an array.
    // The returned value must be positive.
    fn match_cost(self) -> f32;

    // Return whether the current valid doc in the approximating DocSet is on a match.
    // This should only be called when the DocSet is positioned, and at most once.
    // The approximating DocSet implements the first phase, this method implements the second phase.
    fn matches(&mut self) -> bool;
}

impl<TDocSet: DocSet> DocSet for TwoPhaseApproximation<TDocSet> {
    // Much like ConstScorer in scorer. CHECKME: avoid this almost duplication?
    fn advance(&mut self) -> bool {
        self.approximation.advance()
    }

    fn skip_next(&mut self, target: DocId) -> SkipResult {
        self.approximation.skip_next(target)
    }

    fn doc(&self) -> DocId {
        self.approximation.doc()
    }

    fn fill_buffer(&mut self, buffer: &mut [DocId]) -> usize {
        self.approximation.fill_buffer(buffer)
    }

    fn size_hint(&self) -> u32 {
        self.approximation.size_hint()
    }

    fn append_to_bitset(&mut self, bitset: &mut BitSet) {
        self.approximation.append_to_bitset(bitset);
    }
}
