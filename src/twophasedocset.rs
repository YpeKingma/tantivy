
mistake

struct TwoPhaseDocSet<TDocSet: DocSet> {
   approximation: TDocSet,
}

impl TwoPhaseDocSet<TDocSet: DocSet> {
    pub fn new(approximation: TDocSet) -> TwoPhaseDocSet<TDocSet> {
        TwoPhaseDoc<TDocSet> {
            approximation
        }
    }

    pub fn approximation(self) -> DocSet {
        self.approximation
    }
}

pub trait TwoPhaseDocSet {
    // An estimate of the expected cost to determine that a single document `.matches()`.
    // Returns an expected cost in number of simple operations like addition, multiplication,
    // comparing two numbers and indexing an array.
    // The returned value must be positive.
    pub fn match_cost() -> f32;

    // Return whether the current valid doc in the approximating DocSet is on a match.
    // This should only be called when the DocSet is positioned, and at most once.
    // The approximating DocSet implements the first phase, this method implements the second phase.
    pub fn matches(&mut self) -> bool;
}


impl DocSet for TwoPhaseDocSet<TDocSet: DocSet> {
    fn advance(&mut self) -> bool {
        self.approximation.advance()
    }

    fn skip_next(&mut self, target: DocId) -> SkipResult {
        self.approximation.skip_next(target)
    }
}
