// oraide - tools for OpenRA-based mod/game development
// get the source code at https://github.com/Phrohdoh/oraide
//
// copyright (c)
// - 2020 Taryn "Phrohdoh" Hill

#![deny(missing_docs)]

//! This [module] exposes types regarding deriving spanned-lines from a
//! [MiniYaml document], including a trait ([Spanner]) which allows consumers
//! to supply their own implementation, and a convenience function
//! (`span_lines_of`) which uses an implementor of said trait, [DefaultSpanner].
//!
//! [module]: https://doc.rust-lang.org/book/ch07-02-defining-modules-to-control-scope-and-privacy.html
//! [MiniYaml document]: struct.MiniYamlDoc.html
//! [Spanner]: trait.Spanner.html
//! [convenience function]: trait.Spanner.html
//! [DefaultSpanner]: struct.DefaultSpanner.html

use {
    std::{
        str::CharIndices,
        iter::Peekable,
    },
    crate::{
        AbsByteIdx,
        AbsByteIdxSpan,
    },
};

// ----- public interface ------------------------------------------------------

/// The ability to produce spanned lines.
pub trait Spanner {
    /// Derive [`SpannedLine`]s via the stateful implementor.
    ///
    /// [`SpannedLine`]: struct.SpannedLine.html
    fn span_lines(&mut self) -> SpannedLines;
}

/// A componentized line of text.
///
/// [`raw`] spans the entire line (including leading and trailing whitespace,
/// i.e., line-terminator).
///
/// [`raw`]: struct.SpannedLine.html#structfield.raw
#[derive(PartialEq)]
#[cfg_attr(test, derive(Debug))]
pub struct SpannedLine {
    /// absolutely-positioned span of the entire line
    /// (spans over all the other fields)
    pub raw: AbsByteIdxSpan,

    /// absolutely-positioned span of the line's indentation, if it exists
    pub indent: Option<AbsByteIdxSpan>,

    /// absolutely-positioned span of the line's key, if it exists
    pub key: Option<AbsByteIdxSpan>,

    /// absolutely-positioned span of the line's key-separator, if it exists,
    /// which isn't required to have a key, but is required to have a value
    pub key_sep: Option<AbsByteIdxSpan>,

    /// absolutely-positioned span of the line's "value", if it exists
    /// (MiniYaml is comprised of key-value lines)
    pub value: Option<AbsByteIdxSpan>,

    /// absolutely-positioned span of the line's comment, if it exists
    pub comment: Option<AbsByteIdxSpan>,

    /// absolutely-positioned span of the line's terminator, if it exists
    pub term: Option<AbsByteIdxSpan>,
}

/// Derive spanned-lines from `doc` via [`DefaultSpanner`].
///
/// [`DefaultSpanner`]: struct.DefaultSpanner.html
pub fn span_lines_of(doc: &str) -> SpannedLines {
    let mut spanner = DefaultSpanner::new(doc);
    spanner.span_lines()
}

// ----- private implementation details ----------------------------------------

type SpannedLines = Vec<SpannedLine>;

/// Our implementation of MiniYaml spanning is a 2-stage process:
///
/// raw UTF-8 text -> 0+ `RawAndTerm`s -> 0+ `SpannedLine`
///
/// This type, `RawAndTerm`, is a private, intermediate type used to make the
/// implementation easier to comprehend than it would be with tuples everywhere.
struct RawAndTerm {
    raw: AbsByteIdxSpan,
    term: Option<AbsByteIdxSpan>,
}

impl RawAndTerm {
    fn logical_line_end_abx(&self) -> AbsByteIdx {
        self.term.map(|term| term.start)
            .unwrap_or(self.raw.end)
    }
}

struct DefaultSpanner<'doc> {
    _doc: &'doc str,
    _doc_len_bytes: usize,
}

impl<'doc> DefaultSpanner<'doc> {
    fn new(doc: &'doc str) -> Self {
        Self {
            _doc_len_bytes: doc.len(),
            _doc: doc,
        }
    }

    #[inline(always)]
    fn is_empty(&self) -> bool {
        self._doc_len_bytes == 0
    }

    #[inline(always)]
    fn iter(&self) -> Peekable<CharIndices> {
        let ch_idx_iter = self._doc.char_indices().peekable();
        ch_idx_iter
    }

    #[track_caller]
    fn text_at(&self, abs_start_idx_li: usize, abs_end_idx_he: usize) -> &str {
        assert!(
            abs_end_idx_he <= self._doc_len_bytes,
            "attempted to read past end of doc's contents: [{}..{})",
            abs_start_idx_li,
            abs_end_idx_he,
        );

        &self._doc[abs_start_idx_li..abs_end_idx_he]
    }

    fn lines(&self) -> Vec<RawAndTerm> {
        if self.is_empty() {
            return vec![];
        }

        let mut ret = vec![];

        let mut ch_idx_iter = self.iter();
        let mut line_start_abx = 0.into(); // start at beginning of document

        while let Some((ch_start_abs_idx, ch)) = ch_idx_iter.next() {
            let ch_start_abx = AbsByteIdx(ch_start_abs_idx);
            let ch_end_abx = (ch_start_abs_idx + ch.len_utf8()).into();

            let opt_next_tup2 = ch_idx_iter.peek()
                .map(|tup2| tup2.to_owned());

            let opt_term_span = match ch {
                '\n' => Some((ch_start_abx, ch_end_abx).into()),
                '\r' => match opt_next_tup2 {
                    Some((lf_start_abs_idx, '\n')) => {
                        // advance over the `\n`
                        ch_idx_iter.next();

                        let lf_end_idx = lf_start_abs_idx + '\n'.len_utf8();
                        let end_abx = lf_end_idx.into();
                        let abx_span = AbsByteIdxSpan::from((ch_start_abx, end_abx));
                        Some(abx_span)
                    },
                    _ => Some((ch_start_abx, ch_end_abx).into())
                }
                _ => None,
            };

            let (raw_span, term_span) = match (opt_term_span, opt_next_tup2) {
                // terminating the current line, with another line following
                (Some(term_span), Some(_)) => {
                    let raw_span = AbsByteIdxSpan::from((
                        line_start_abx,
                        term_span.end,
                    ));

                    line_start_abx = term_span.end;

                    (raw_span, term_span.into())
                },
                // end-of-document with trailing line-terminator
                (Some(term_span), None) => {
                    let raw_span = (line_start_abx, term_span.end).into();
                    (raw_span, term_span.into())
                },
                // abrupt end-of-document (no trailing line-terminator)
                (None, None) => {
                    let raw_span = (line_start_abx, ch_end_abx).into();
                    (raw_span, None)
                }
                // not a line-terminator, followed by something
                (None, Some(_)) => continue,
            };

            ret.push(
                RawAndTerm {
                    raw: raw_span,
                    term: term_span,
                }
            );
        }

        ret
    }

    /// if spanning `hello\n`, `line_txt` would be `hello`
    fn componentize_line(
        &self,
        raw_and_term: RawAndTerm,
        line_txt: &'doc str,
    ) -> SpannedLine {
        let RawAndTerm { raw, term } = raw_and_term;

        let mut ret = SpannedLine {
            raw,
            indent: Default::default(),
            key: Default::default(),
            key_sep: Default::default(),
            value: Default::default(),
            comment: Default::default(),
            term,
        };

        let is_line_empty = line_txt.is_empty();
        if is_line_empty {
            return ret;
        }

        let line_start_abx = raw.start;
        let logical_line_end_abx = raw_and_term.logical_line_end_abx();

        let is_indent_only = line_txt.trim().is_empty();
        if is_indent_only {
            let indent_span = AbsByteIdxSpan::from((
                line_start_abx,
                logical_line_end_abx,
            ));

            ret.indent = indent_span.into();
            return ret;
        }

        // at this point the easy cases have been handled, so we move onto the
        // more complex portion of componentizing

        /// a byte index relative to the current line's absolute start position
        #[derive(Debug, Copy, Clone, PartialEq)]
        struct RelByteIdx(usize);

        impl RelByteIdx {
            #[inline(always)]
            fn is_start_of_line(&self) -> bool {
                self.0 == 0
            }
        }

        impl From<usize> for RelByteIdx {
            fn from(rel_idx: usize) -> Self {
                Self(rel_idx)
            }
        }

        impl std::ops::Add<usize> for RelByteIdx {
            type Output = RelByteIdx;

            fn add(self, rel_idx: usize) -> Self::Output {
                Self(self.0 + rel_idx)
            }
        }

        impl std::ops::Sub<usize> for RelByteIdx {
            type Output = RelByteIdx;

            fn sub(self, rel_idx: usize) -> Self::Output {
                Self(self.0 - rel_idx)
            }
        }

        impl std::ops::Add<RelByteIdx> for AbsByteIdx {
            type Output = Self;

            fn add(self, rbx: RelByteIdx) -> Self::Output {
                Self(self.0 + rbx.0)
            }
        }

        // convert a line-relative byte index into an absolutely-positioned byte index
        let rbx_to_abx = |rbx: RelByteIdx| -> AbsByteIdx {
            raw.start + rbx
        };

        let (first_non_ws_rbx, first_non_ws_abx) = {
            let rbx = line_txt.find(|c: char| !c.is_whitespace())
                .map(Into::into)
                .unwrap(/* safe due to previous `return`s */);

            let abx = rbx_to_abx(rbx);

            (rbx, abx)
        };

        // we now have all of the information we need to set the indent
        ret.indent = if first_non_ws_rbx.is_start_of_line() {
            None
        } else {
            Some((line_start_abx, first_non_ws_abx).into())
        };

        // TODO: test escaped comment followed by actual comment
        let opt_first_comment_start_bxs = line_txt.find('#')
            .map(|ridx| {
                let rbx = ridx.into();
                let abx = rbx_to_abx(rbx);
                (rbx, abx)
            });

        let opt_comment_span = opt_first_comment_start_bxs
            .and_then(|(comment_start_rbx, comment_start_abx)| {
                if comment_start_rbx == first_non_ws_rbx {
                    // entire line from this point onward is a comment
                    return Some((comment_start_abx, logical_line_end_abx).into())
                }

                // TODO: possibly not a valid boundary
                let range_of_txt_preceeding_comment_start = (comment_start_rbx.0) - 1 .. (comment_start_rbx.0);
                let opt_txt_preceeding_comment_start = line_txt.get(range_of_txt_preceeding_comment_start);
                let is_escaped_comment_start = opt_txt_preceeding_comment_start
                    .map(|txt| txt.starts_with('\\'))
                    .unwrap_or(false);

                if is_escaped_comment_start {
                    return None;
                }

                Some((comment_start_abx, logical_line_end_abx).into())
            });

        // we now have all of the information we need to set the comment,
        // sans the escaped comment TODO
        ret.comment = opt_comment_span;

        let opt_key_sep_span = {
            // end at the comment, if one exists
            let find_end_abs_idx = opt_comment_span
                .map(|comment_span| comment_span.start)
            // otherwise search til end-of-line
                .unwrap_or(logical_line_end_abx)
            // we're going to slice with this, so need the inner `usize`
                .0;

            let find_end_ridx = find_end_abs_idx - line_start_abx.0;
            let line_txt_preceeding_comment = &line_txt[..find_end_ridx];
            let opt_key_sep_start_rbx = line_txt_preceeding_comment.find(':')
                .map(|ridx| RelByteIdx::from(ridx));

            if let Some(key_sep_start_rbx) = opt_key_sep_start_rbx {
                let start_abx = rbx_to_abx(key_sep_start_rbx);
                let end_rbx = (key_sep_start_rbx.0 + ':'.len_utf8()).into();
                let end_abx = rbx_to_abx(end_rbx);

                AbsByteIdxSpan::from((
                    start_abx,
                    end_abx,
                )).into()
            } else {
                None
            }
        };

        // we now have all of the information we need to set the comment
        ret.key_sep = opt_key_sep_span;

        let opt_key_span =
            if ret.comment.map(|span| span.start) == Some(first_non_ws_abx) {
                // line is either comment-only or indentation then comment
                None
            } else {
                match (opt_key_sep_span, opt_comment_span) {
                    (Some(key_sep_span), Some(comment_span)) => {
                        let key_end_abs_idx = ((key_sep_span.start).0).min((comment_span.start).0);
                        let key_end_abx = AbsByteIdx::from(key_end_abs_idx);

                        AbsByteIdxSpan::from((
                            first_non_ws_abx,
                            key_end_abx,
                        ))
                    },
                    (Some(key_sep_span), None) => AbsByteIdxSpan::from((
                        first_non_ws_abx,
                        key_sep_span.start,
                    )),
                    (None, Some(comment_span)) => AbsByteIdxSpan::from((
                        first_non_ws_abx,
                        comment_span.start,
                    )),
                    (None, None) => AbsByteIdxSpan::from((
                        first_non_ws_abx,
                        logical_line_end_abx,
                    )),
                }.into()
            };

        ret.key = opt_key_span;

        let opt_value_span = opt_key_sep_span.and_then(|key_sep_span| {
            let key_sep_end_ridx = key_sep_span.end.0 - line_start_abx.0;
            let line_txt_after_key_sep = &line_txt[key_sep_end_ridx..];

            let opt_value_start_ridx = line_txt_after_key_sep.find(|c: char| !c.is_whitespace())
                .map(|rel_rel_idx| rel_rel_idx + key_sep_end_ridx);

            match (opt_value_start_ridx, opt_comment_span) {
                (Some(value_start_ridx), Some(comment_span)) => {
                    let comment_start_abx = comment_span.start;
                    let comment_start_ridx = comment_start_abx.0;
                    let value_start_abs_idx = line_start_abx.0 + value_start_ridx;

                    if value_start_ridx < comment_start_ridx {
                        AbsByteIdxSpan::from((
                            value_start_abs_idx.into(),
                            comment_start_abx,
                        )).into()
                    } else {
                        None
                    }
                },
                (Some(value_start_ridx), None) => AbsByteIdxSpan::from((
                    AbsByteIdx(line_start_abx.0 + value_start_ridx),
                    logical_line_end_abx
                )).into(),
                _ => None,
            }
        });

        ret.value = opt_value_span;

        ret
    }
}

impl Spanner for DefaultSpanner<'_> {
    fn span_lines(&mut self) -> SpannedLines {
        let lines = self.lines();
        let spanned_lines = lines.into_iter()
            .map(|raw_and_term| {
                let line_txt = {
                    let abs_start_idx = raw_and_term.raw.start.0;

                    // end at whichever is first, line-term start or raw's end
                    let abs_end_idx = raw_and_term.term.map(|term| term.start.0)
                        .unwrap_or(raw_and_term.raw.end.0);

                    self.text_at(abs_start_idx, abs_end_idx)
                };

                self.componentize_line(raw_and_term, line_txt)
            })
            .collect::<Vec<_>>();

        spanned_lines
    }
}

// ----- tests -----------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// a helper which wraps the given string slice in a `MiniYamlDoc`
    fn span_lines_of(doc_txt: &str) -> SpannedLines {
        super::span_lines_of(doc_txt.into())
    }

    macro_rules! assert_eq_span {
        (
            expected: $expected:expr,
            actual: $actual:expr,
            $debug_hint_fmt:expr,
            $($debug_hint_arg:tt)+
        ) => {
            let expected_tup2 = Into::<(_, _)>::into($expected);
            let actual_tup2 = Into::<(_, _)>::into($actual);

            assert_eq!(
                expected_tup2,
                actual_tup2,
                "{}", format_args!($debug_hint_fmt, $($debug_hint_arg)+),
            );
        };
    }

    macro_rules! assert_eq_opt_span {
        (
            expected: $expected:expr,
            actual: $actual:expr,
            $debug_hint_fmt:expr,
            $($debug_hint_arg:tt)+
        ) => {
            let expected_opt_tup2 = $expected.map(|span| Into::<(_, _)>::into(span));
            let actual_opt_tup2 = $actual.map(|span| Into::<(_, _)>::into(span));

            assert_eq!(
                expected_opt_tup2,
                actual_opt_tup2,
                "{}", format_args!($debug_hint_fmt, $($debug_hint_arg)+),
            );
        };
    }

    macro_rules! piecewise_assert_eq_spanned_lines {
        (
            expected: $expected:expr,
            actual: $actual:expr,
            line_num: $line_num:expr
        ) => {
            assert_eq_span!(
                expected: $expected.raw,
                actual: $actual.raw,
                "line {} raw",
                $line_num
            );

            assert_eq_opt_span!(
                expected: $expected.indent,
                actual: $actual.indent,
                "line {} indent",
                $line_num
            );

            assert_eq_opt_span!(
                expected: $expected.key,
                actual: $actual.key,
                "line {} key",
                $line_num
            );

            assert_eq_opt_span!(
                expected: $expected.key_sep,
                actual: $actual.key_sep,
                "line {} key_sep",
                $line_num
            );

            assert_eq_opt_span!(
                expected: $expected.value,
                actual: $actual.value,
                "line {} value",
                $line_num
            );

            assert_eq_opt_span!(
                expected: $expected.comment,
                actual: $actual.comment,
                "line {} comment",
                $line_num
            );

            assert_eq_opt_span!(
                expected: $expected.term,
                actual: $actual.term,
                "line {} term",
                $line_num
            );
        };
    }

    mod simple {
        use super::*;

        #[test]
        fn empty_doc_returns_empty_collection() {
            // arrange
            let doc = "";

            // act
            let actual_spanned_lines = span_lines_of(doc);

            // assert
            assert!(actual_spanned_lines.is_empty());
        }

        #[test]
        fn whitespace_only_lines() {
           // arrange
           let doc = [
               /* line 1: start=0  end=1   */ "\n",
               /* line 2: start=1  end=3   */ "\r\n",
               /* line 3: start=3  end=5   */ " \n",
               /* line 4: start=5  end=8   */ " \r\n",
               /* line 5: start=8  end=10  */ " \r",
               /* line 6: start=10 end=11  */ " ",
           ].join("");

            let expected_lines = vec![
                SpannedLine { // line 1
                    raw: (0, 1).into(),
                    indent: None,
                    key: None,
                    key_sep: None,
                    value: None,
                    comment: None,
                    term: Some((0, 1).into()),
                },
                SpannedLine { // line 2
                    raw: (1, 3).into(),
                    indent: None,
                    key: None,
                    key_sep: None,
                    value: None,
                    comment: None,
                    term: Some((1, 3).into()),
                },
                SpannedLine { // line 3
                    raw: (3, 5).into(),
                    indent: Some((3, 4).into()),
                    key: None,
                    key_sep: None,
                    value: None,
                    comment: None,
                    term: Some((4, 5).into()),
                },
                SpannedLine { // line 4
                    raw: (5, 8).into(),
                    indent: Some((5, 6).into()),
                    key: None,
                    key_sep: None,
                    value: None,
                    comment: None,
                    term: Some((6, 8).into()),
                },
                SpannedLine { // line 5
                    raw: (8, 10).into(),
                    indent: Some((8, 9).into()),
                    key: None,
                    key_sep: None,
                    value: None,
                    comment: None,
                    term: Some((9, 10).into()),
                },
                SpannedLine { // line 6
                    raw: (10, 11).into(),
                    indent: Some((10, 11).into()),
                    key: None,
                    key_sep: None,
                    value: None,
                    comment: None,
                    term: None,
                },
            ];

           // act
           let actual_lines = span_lines_of(&doc);

           // assert
           assert_eq!(
               expected_lines,
               actual_lines,
           );
        }

        #[test]
        fn lf_followed_by_empty_line() {
           // arrange
           let doc = vec![
               "\n",
               "",
           ].join("");

           let expected_lines = vec![
               SpannedLine {
                   raw: (0, 1).into(),
                   indent: None,
                   key: None,
                   key_sep: None,
                   value: None,
                   comment: None,
                   term: Some((0, 1).into()),
               },
           ];

           // act
           let actual_lines = span_lines_of(&doc);

           // assert
           assert_eq!(
               expected_lines,
               actual_lines,
           );
        }

        #[test]
        fn crlf_followed_by_empty_line() {
           // arrange
           let doc = vec![
               "\r\n",
               "",
           ].join("");

           let expected_lines = vec![
               SpannedLine {
                   raw: (0, 2).into(),
                   indent: None,
                   key: None,
                   key_sep: None,
                   value: None,
                   comment: None,
                   term: Some((0, 2).into()),
               },
           ];

           // act
           let actual_lines = span_lines_of(&doc);

           // assert
           assert_eq!(
               expected_lines,
               actual_lines,
           );
        }

        #[test]
        fn cr_followed_by_empty_line() {
           // arrange
           let doc = vec![
               "\r",
               "",
           ].join("");

           let expected_lines = vec![
               SpannedLine {
                   raw: (0, 1).into(),
                   indent: None,
                   key: None,
                   key_sep: None,
                   value: None,
                   comment: None,
                   term: Some((0, 1).into()),
               },
           ];

           // act
           let actual_lines = span_lines_of(&doc);

           // assert
           assert_eq!(
               expected_lines,
               actual_lines,
           );
        }
    }

    #[test]
    fn spanned_line_component_texts() {
       // arrange
       let doc = vec![
           "    hello : world # foo \r",
       ].join("");

       let expected_texts = vec![
            SpannedLineTexts {
                raw:     "    hello : world # foo \r",
                indent:  "    "                       .into(),
                key:         "hello "                 .into(),
                key_sep:           ":"                .into(),
                value:               "world "         .into(),
                comment:                    "# foo "  .into(),
                term:                             "\r".into(),
            },
       ];

       #[derive(Debug, PartialEq)]
       struct SpannedLineTexts<'doc> {
           raw: &'doc str,
           indent: Option<&'doc str>,
           key: Option<&'doc str>,
           key_sep: Option<&'doc str>,
           value: Option<&'doc str>,
           comment: Option<&'doc str>,
           term: Option<&'doc str>,
       }

        let map_opt_span_to_txt = |span: Option<AbsByteIdxSpan>| -> Option<&'_ str> {
            match span {
                None => None,
                Some(span) => {
                    let span_start_abs_idx = span.start.0;
                    let span_end_abs_idx = span.end.0;
                    (&doc[span_start_abs_idx..span_end_abs_idx]).into()
                },
            }
        };

        let spanned_line_to_texts = |line: SpannedLine| -> SpannedLineTexts {
            SpannedLineTexts {
                raw: {
                    let start_idx = line.raw.start.0;
                    let end_idx = line.raw.end.0;
                    &doc[start_idx..end_idx]
                },
                indent: map_opt_span_to_txt(line.indent),
                key: map_opt_span_to_txt(line.key),
                key_sep: map_opt_span_to_txt(line.key_sep),
                value: map_opt_span_to_txt(line.value),
                comment: map_opt_span_to_txt(line.comment),
                term: map_opt_span_to_txt(line.term),
            }
        };

       // act
       let actual_texts = span_lines_of(&doc).into_iter()
           .map(spanned_line_to_texts)
           .collect::<Vec<_>>();

       // assert
       assert_eq!(
           expected_texts,
           actual_texts,
       );
    }

    #[test]
    // test name contains a UUID because I don't have a good name for it
    // and this makes it easily identifiable
    fn span_doc_96106f58_06bf_4e9d_a705_312acd853814() {
        // arrange
        let doc = vec![
            /* 1 */ "E2:\r",
            /* 2 */ "    Inherits:^Soldier\r\n",
            /* 3 */ "    Inherits@experience : ^GainsExperience\n",
            /* 4 */ "	Valued:\r\n",
            /* 5 */ "       # Cost: 300\r",
            /* 6 */ "	    Cost: 200\n",
            /* 7 */ " bar: \\# zoop\n",
        ].join("");

        let expected_spanned_lines = vec![
            SpannedLine { // line 1
                raw: (0, 4).into(),
                indent: None,
                key: Some((0, 2).into()),
                key_sep: Some((2, 3).into()),
                value: None,
                comment: None,
                term: Some((3, 4).into()),
            },
            SpannedLine { // line 2
                raw: (4, 27).into(),
                indent: Some((4, 8).into()),
                key: Some((8, 16).into()),
                key_sep: Some((16, 17).into()),
                value: Some((17, 25).into()),
                comment: None,
                term: Some((25, 27).into()),
            },
            SpannedLine { // line 3
                raw: (27, 70).into(),
                indent: Some((27, 31).into()),
                key: Some((31, 51).into()),
                key_sep: Some((51, 52).into()),
                value: Some((53, 69).into()),
                comment: None,
                term: Some((69, 70).into()),
            },
            SpannedLine { // line 4
                raw: (70, 80).into(),
                indent: Some((70, 71).into()),
                key: Some((71, 77).into()),
                key_sep: Some((77, 78).into()),
                value: None,
                comment: None,
                term: Some((78, 80).into()),
            },
            SpannedLine { // line 5
                raw: (80, 99).into(),
                indent: Some((80, 87).into()),
                key: None,
                key_sep: None,
                value: None,
                comment: Some((87, 98).into()),
                term: Some((98, 99).into()),
            },
            SpannedLine { // line 6
                raw: (99, 114).into(),
                indent: Some((99, 104).into()),
                key: Some((104, 108).into()),
                key_sep: Some((108, 109).into()),
                value: Some((110, 113).into()),
                comment: None,
                term: Some((113, 114).into()),
            },
            SpannedLine { // line 7
                raw: (114, 128).into(),
                indent: Some((114, 115).into()),
                key: Some((115, 118).into()),
                key_sep: Some((118, 119).into()),
                value: Some((120, 127).into()),
                comment: None,
                term: Some((127, 128).into()),
            },
        ];

        // act
        let actual_spanned_lines = span_lines_of(&doc);

        // assert
        assert_eq!(
            expected_spanned_lines.len(),
            actual_spanned_lines.len(),
            "count of `SpannedLine`s",
        );

        let iter_tup2 = expected_spanned_lines.into_iter()
            .zip(actual_spanned_lines.into_iter());

        for (line_idx, (expected_comp_spanned_line, actual_comp_spanned_line)) in iter_tup2.enumerate() {
            piecewise_assert_eq_spanned_lines!(
                expected: expected_comp_spanned_line,
                actual: actual_comp_spanned_line,
                line_num: line_idx + 1
            );
        }
    }

    #[test]
    #[allow(non_snake_case /* referencing types */)]
    fn AbsByteIdx_impls_Index_for_str_correctly() {
       // arrange
       let doc = "     hello: world # welcome ";
       let expected_texts = vec! [ "hello" ];

       // act
       let actual_texts = span_lines_of(doc).into_iter()
        .map(|spanned_line| &doc[spanned_line.key.expect("key spanning broken")])
        .collect::<Vec<_>>();

       // assert
       assert_eq!(
           expected_texts,
           actual_texts,
       );
    }

    #[test]
    #[allow(non_snake_case /* referencing types */)]
    fn AbsByteIdx_impls_Index_for_String_correctly() {
       // arrange
       let doc = "     hello: world # welcome ".to_owned();
       let expected_texts = vec! [ "hello" ];

       // act
       let actual_texts = span_lines_of(&doc).into_iter()
        .map(|spanned_line| &doc[spanned_line.key.expect("key spanning broken")])
        .collect::<Vec<_>>();

       // assert
       assert_eq!(
           expected_texts,
           actual_texts,
       );
    }
}
