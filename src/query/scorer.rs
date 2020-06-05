use crate::docset::{DocSet, TERMINATED};
use crate::query::twophase::TwoPhase;
use crate::DocId;
use crate::Score;
use downcast_rs::impl_downcast;
use std::cell::RefCell;
use std::ops::DerefMut;
use std::rc::Rc;

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
            dbg!("two_phase Some");
            let mut doc = self.doc();
            while doc != TERMINATED {
                if two_phase.matches() {
                    callback(doc, self.score());
                }
                doc = self.advance();
            }
        } else {
            dbg!("two_phase None");
            let mut doc = self.doc();
            while doc != TERMINATED {
                callback(doc, self.score());
                doc = self.advance();
            }
            todo!("two_phase false should not occur for test_phrase_query_docfreq_order");
        }
    }

    /// Calls `callback` with all of the `(doc, score)` for which score
    /// is exceeding a given threshold.
    ///
    /// This method is useful for the TopDocs collector.
    /// For all docsets, the blanket implementation has the benefit
    /// of prefiltering (doc, score) pairs, avoiding the
    /// virtual dispatch cost.
    ///
    /// More importantly, it makes it possible for scorers to implement
    /// important optimization (e.g. BlockWAND for union).
    fn for_each_pruning(
        &mut self,
        mut threshold: f32,
        callback: &mut dyn FnMut(DocId, Score) -> Score,
    ) {
        let mut doc = self.doc();
        while doc != TERMINATED {
            let score = self.score();
            if score > threshold {
                threshold = callback(doc, score);
            }
            doc = self.advance();
        }
        todo!("FIXME: Add two phase as in Scorer.for_each()");
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
    fn two_phase(&mut self) -> Option<Box<dyn TwoPhase>> {
        todo!("Scorer.two_phase() should be overriden in test_phrase_query_docfreq_order");
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

    fn two_phase(&mut self) -> Option<Box<dyn TwoPhase>> {
        self.deref_mut().two_phase()
    }
}

pub struct RcRefCellScorer<TScorer: Scorer>(Rc<RefCell<TScorer>>);

impl<TScorer: Scorer> RcRefCellScorer<TScorer> {
    pub fn new(scorer: TScorer) -> Self {
        RcRefCellScorer(Rc::new(RefCell::new(scorer)))
    }

    pub fn scorer_is<T>(self) -> bool {
        self.0.as_ref().borrow().is::<T>()
    }
}

/// Provides Scorer methods by using a borrowed scorer
pub trait BorrowScorer<TScorer: Scorer>: Scorer {

    fn scorer_borrow(&self) -> &TScorer; // Implement by self.0.as_ref().borrow() on RcRefCellScorer

    fn scorer_borrow_mut(&mut self) -> &mut TScorer; // Similarly by self.0.as_ref().borrow_mut()

    fn scorer_borrow_tomut(&self) -> &mut TScorer {
        self.scorer_borrow_mut()
    }

    fn score(&mut self) -> Score {
        self.scorer_borrow_mut().score()
    }

    fn for_each(&mut self, callback: &mut dyn FnMut(DocId, Score)) {
        self.scorer_borrow_mut().for_each(callback)
    }

    fn two_phase(&mut self) -> Option<Box<dyn TwoPhase>> {
        self.scorer_borrow_mut().two_phase()
    }

    fn doc(&self) -> DocId {
        self.scorer_borrow().doc()
    }

    fn size_hint(&self) -> u32 {
        self.scorer_borrow().size_hint()
    }

    fn advance(&mut self) -> DocId {
        self.scorer_borrow_mut().advance()
    }

    fn seek(&mut self, doc: DocId) -> DocId {
        self.scorer_borrow_mut().seek(doc)
    }
}

/// Provide scorer borrow functions of BorrowScorer for RcRefCellScorer(scorer_type)
macro_rules! impl_scorer_rc_borrow {
    ($scorer_type:ident) => {
        fn scorer_borrow(&self) -> &$scorer_type {
            self.0.as_ref().borrow()
        }

        fn scorer_borrow_mut(&mut self) -> &mut $scorer_type {
            self.0.as_ref().borrow_mut()
        }
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
    fn advance(&mut self) -> DocId {
        self.docset.advance()
    }

    fn seek(&mut self, target: DocId) -> DocId {
        self.docset.seek(target)
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
}

impl<TDocSet: DocSet + 'static> Scorer for ConstScorer<TDocSet> {
    fn score(&mut self) -> Score {
        self.score
    }
}
