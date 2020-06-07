use crate::docset::DocSet;
use crate::fastfield::DeleteBitSet;
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
}

impl_downcast!(Scorer);

impl Scorer for Box<dyn Scorer> {
    fn score(&mut self) -> Score {
        self.deref_mut().score()
    }
}

impl Scorer for Rc<RefCell<dyn Scorer>> {
    fn score(&mut self) -> Score {
        self.as_ref().borrow_mut().score()
    }
}

impl DocSet for Rc<RefCell<dyn Scorer>> {
    fn advance(&mut self) -> DocId {
        self.as_ref().borrow_mut().advance()
    }

    fn seek(&mut self, target: DocId) -> DocId {
        self.as_ref().borrow_mut().seek(target)
    }

    fn doc(&self) -> DocId {
        self.as_ref().borrow().doc()
    }

    fn size_hint(&self) -> u32 {
        self.as_ref().borrow().size_hint()
    }

    fn count(&mut self, delete_bitset: &DeleteBitSet) -> u32 {
        self.as_ref().borrow().count(delete_bitset)
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
