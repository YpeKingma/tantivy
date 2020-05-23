use crate::common::BitSet;
use crate::docset::{DocSet, SkipResult};
use crate::query::twophase::TwoPhase;
use crate::DocId;
use crate::Score;
use downcast_rs::impl_downcast;
use std::ops::DerefMut;
use std::rc::Rc;
use std::cell::RefCell;

/// Scored set of documents matching a query within a specific segment.
///
/// See [`Query`](./trait.Query.html).
pub trait Scorer: downcast_rs::Downcast + DocSet + 'static {
    /// Returns the score.
    ///
    /// This method will perform a bit of computation and is not cached.
    fn score(&mut self) -> Score;

    /// Iterates through all of the document matched by the DocSet
    /// `DocSet`, that are also indicated as matching by the TwoPhase when available,
    /// and push the scored documents to the collector.
    fn for_each(&mut self, callback: &mut dyn FnMut(DocId, Score)) {
        if let Some(mut two_phase) = self.two_phase() {
            while self.advance() {
                if two_phase.matches() {
                    callback(self.doc(), self.score());
                }
            }
        } else {
            while self.advance() {
                callback(self.doc(), self.score());
            }
        }
    }

    /// Return a TwoPhase for this Scorer, when available.
    ///
    /// Note that the approximation DocSet for the TwoPhase is
    /// the Scorer itself.
    ///
    /// Implementing this method is typically useful on a Scorer
    /// that has a high per-document overhead for confirming matches.
    ///
    /// This implementation returns None.
    fn two_phase(&mut self) -> Option<Rc<RefCell<dyn TwoPhase>>> {
        None
    }
}

impl_downcast!(Scorer);

impl Scorer for Box<dyn Scorer> {
    fn score(&mut self) -> Score {
        self.deref_mut().score()
    }

    fn for_each(&mut self, callback: &mut dyn FnMut(DocId, Score)) {
        let scorer = self.deref_mut();
        scorer.for_each(callback);
    }

    fn two_phase(&mut self) -> Option<Rc<RefCell<dyn TwoPhase>>> {
        self.deref_mut().two_phase()
    }
}

/// Wraps a `DocSet` and simply returns a constant `Scorer`.
/// The `ConstScorer` is useful if you have a `DocSet` where
/// you needed a scorer.
///
/// The `ConstScorer`'s constant score can be set
/// by calling `.set_score(...)`.
pub struct ConstScorer<TDocSet: DocSet> {
    docset: TDocSet,
    score: Score,
}

impl<TDocSet: DocSet> ConstScorer<TDocSet> {
    /// Creates a new `ConstScorer`.
    pub fn new(docset: TDocSet, score: f32) -> ConstScorer<TDocSet> {
        ConstScorer { docset, score }
    }
}

impl<TDocSet: DocSet> From<TDocSet> for ConstScorer<TDocSet> {
    fn from(docset: TDocSet) -> Self {
        ConstScorer::new(docset, 1.0f32)
    }
}

impl<TDocSet: DocSet> DocSet for ConstScorer<TDocSet> {
    fn advance(&mut self) -> bool {
        self.docset.advance()
    }

    fn skip_next(&mut self, target: DocId) -> SkipResult {
        self.docset.skip_next(target)
    }

    fn fill_buffer(&mut self, buffer: &mut [DocId]) -> usize {
        self.docset.fill_buffer(buffer)
    }

    fn doc(&self) -> DocId {
        self.docset.doc()
    }

    fn size_hint(&self) -> u32 {
        self.docset.size_hint()
    }

    fn append_to_bitset(&mut self, bitset: &mut BitSet) {
        self.docset.append_to_bitset(bitset);
    }
}

impl<TDocSet: DocSet + 'static> Scorer for ConstScorer<TDocSet> {
    fn score(&mut self) -> Score {
        self.score
    }
}
