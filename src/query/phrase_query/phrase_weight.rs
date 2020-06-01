use super::PhraseScorer;
use crate::core::SegmentReader;
use crate::fieldnorm::FieldNormReader;
use crate::postings::SegmentPostings;
use crate::query::bm25::BM25Weight;
use crate::query::explanation::does_not_match;
use crate::query::Scorer;
use crate::query::Weight;
use crate::query::{EmptyScorer, Explanation};
use crate::schema::IndexRecordOption;
use crate::schema::Term;
use crate::Result;
use crate::{DocId, DocSet};
use crate::query::scorer::RcRefCellScorer;


pub struct PhraseWeight {
    phrase_terms: Vec<(usize, Term)>,
    similarity_weight: BM25Weight,
    score_needed: bool,
}

impl PhraseWeight {
    /// Creates a new phrase weight.
    pub fn new(
        phrase_terms: Vec<(usize, Term)>,
        similarity_weight: BM25Weight,
        score_needed: bool,
    ) -> PhraseWeight {
        PhraseWeight {
            phrase_terms,
            similarity_weight,
            score_needed,
        }
    }

    fn fieldnorm_reader(&self, reader: &SegmentReader) -> FieldNormReader {
        let field = self.phrase_terms[0].1.field();
        reader.get_fieldnorms_reader(field)
    }

    fn phrase_scorer(
        &self,
        reader: &SegmentReader,
        boost: f32,
    ) -> Result<Option<PhraseScorer<SegmentPostings>>> {
        let similarity_weight = self.similarity_weight.boost_by(boost);
        let fieldnorm_reader = self.fieldnorm_reader(reader);
        if reader.has_deletes() {
            let mut term_postings_list = Vec::new();
            for &(offset, ref term) in &self.phrase_terms {
                if let Some(postings) = reader
                    .inverted_index(term.field())
                    .read_postings(&term, IndexRecordOption::WithFreqsAndPositions)
                {
                    term_postings_list.push((offset, postings));
                } else {
                    return Ok(None);
                }
            }
            Ok(Some(PhraseScorer::new(
                term_postings_list,
                similarity_weight,
                fieldnorm_reader,
                self.score_needed,
            )))
        } else {
            let mut term_postings_list = Vec::new();
            for &(offset, ref term) in &self.phrase_terms {
                if let Some(postings) = reader
                    .inverted_index(term.field())
                    .read_postings_no_deletes(&term, IndexRecordOption::WithFreqsAndPositions)
                {
                    term_postings_list.push((offset, postings));
                } else {
                    return Ok(None);
                }
            }
            Ok(Some(PhraseScorer::new(
                term_postings_list,
                similarity_weight,
                fieldnorm_reader,
                self.score_needed,
            )))
        }
    }
}

impl Weight for PhraseWeight {
    fn scorer(&self, reader: &SegmentReader, boost: f32) -> Result<RcRefCellScorer> {
        Ok(RcRefCellScorer::new(
            if let Some(scorer) = self.phrase_scorer(reader, boost)? {
                scorer
            } else {
                EmptyScorer
            }
        ))
    }

    fn explain(&self, reader: &SegmentReader, doc: DocId) -> Result<Explanation> {
        let scorer_opt = self.phrase_scorer(reader, 1.0f32)?;
        if scorer_opt.is_none() {
            return Err(does_not_match(doc));
        }
        let mut scorer = scorer_opt.unwrap();
        if scorer.seek(doc) != doc {
            return Err(does_not_match(doc));
        }
        let fieldnorm_reader = self.fieldnorm_reader(reader);
        let fieldnorm_id = fieldnorm_reader.fieldnorm_id(doc);
        let phrase_count = scorer.phrase_count();
        let mut explanation = Explanation::new("Phrase Scorer", scorer.score());
        explanation.add_detail(self.similarity_weight.explain(fieldnorm_id, phrase_count));
        Ok(explanation)
    }
}

#[cfg(test)]
mod tests {
    use super::super::phrase_scorer::RcRefCellPhraseScorer;
    use super::super::tests::create_index;
    use crate::docset::TERMINATED;
    use crate::query::PhraseQuery;
    use crate::{DocSet, Term};

    use crate::query::Scorer;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[test]
    pub fn test_phrase_count() {
        let index = create_index(&["a c", "a a b d a b c", " a b"]);
        let schema = index.schema();
        let text_field = schema.get_field("text").unwrap();
        let searcher = index.reader().unwrap().searcher();
        let phrase_query = PhraseQuery::new(vec![
            Term::from_field_text(text_field, "a"),
            Term::from_field_text(text_field, "b"),
        ]);
        let phrase_weight = phrase_query.phrase_weight(&searcher, true).unwrap();
        let mut phrase_scorer = RcRefCellPhraseScorer::new(
            phrase_weight
                .phrase_scorer(searcher.segment_reader(0u32), 1.0f32)
                .unwrap()
                .unwrap(),
        );
        let mut phrase_two_phase = phrase_scorer.two_phase().unwrap();
        assert_eq!(phrase_scorer.doc(), 1);
        assert!(phrase_two_phase.matches());
        assert_eq!(phrase_scorer.phrase_count(), 2);
        assert_eq!(phrase_scorer.advance(), 2);
        assert_eq!(phrase_scorer.doc(), 2);
        assert!(phrase_two_phase.matches());
        assert_eq!(phrase_scorer.phrase_count(), 1);
        assert_eq!(phrase_scorer.advance(), TERMINATED);
    }
}
