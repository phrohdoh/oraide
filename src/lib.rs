// oraide - tools for OpenRA-based mod/game development
// get the source code at https://github.com/Phrohdoh/oraide
//
// copyright (c)
// - 2020 Taryn "Phrohdoh" Hill

#[derive(Debug, PartialEq)]
pub struct ComponentizedLine {
    pub indent: Option<ByteIdxSpan>,
    pub key: Option<ByteIdxSpan>,
    pub key_sep: Option<ByteIdxSpan>,
    pub value: Option<ByteIdxSpan>,
    pub comment: Option<ByteIdxSpan>,
    pub term: Option<ByteIdxSpan>,
}

/// 0-based
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct ByteIdx(usize);

impl ByteIdx {
    pub fn as_inner(&self) -> usize {
        self.0
    }
}

/// low-inclusive, high-exclusive byte index span
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct ByteIdxSpan(
    ByteIdx,
    ByteIdx,
);

impl std::ops::Index<ByteIdxSpan> for String {
    type Output = str;

    fn index(&self, index: ByteIdxSpan) -> &Self::Output {
        let start_abs_idx = (index.0).0;
        let end_abs_idx = (index.1).0;
        &self[start_abs_idx..end_abs_idx]
    }
}

impl From<usize> for ByteIdx {
    fn from(idx: usize) -> Self {
        Self(idx)
    }
}

impl From<(usize, usize)> for ByteIdxSpan {
    fn from((start, end): (usize, usize)) -> Self {
        Self(start.into(), end.into())
    }
}

impl From<(ByteIdx, ByteIdx)> for ByteIdxSpan {
    fn from((start, end): (ByteIdx, ByteIdx)) -> Self {
        Self(start, end)
    }
}

impl From<ByteIdxSpan> for std::ops::Range<usize> {
    fn from(bx_span: ByteIdxSpan) -> Self {
        Self {
            start: (bx_span.0).0,
            end: (bx_span.1).0,
        }
    }
}

impl From<ByteIdxSpan> for (usize, usize) {
    fn from(bx_span: ByteIdxSpan) -> Self {
        (
            (bx_span.0).0,
            (bx_span.1).0,
        )
    }
}

/// A document composed of the following "raw" lines
/// ```plaintext
/// hello\n
/// world\r\n
/// how are you?
/// ```
/// Is represented by the following `RawSpannedLine`s
/// ```plaintext
/// { raw: ( 0,  6), term: Some(( 5,  6)) }
/// { raw: ( 6, 13), term: Some((11, 13)) }
/// { raw: (13, 24), term: None           }
/// ```
#[derive(Debug, PartialEq, Clone)]
struct IntermediateRawSpannedLine {
    raw: ByteIdxSpan,
    term: Option<ByteIdxSpan>,
}

impl IntermediateRawSpannedLine {
    fn line_end(&self) -> ByteIdx {
        self.term.map(|term_span| term_span.0)
            .unwrap_or(self.raw.1)
    }
}

/// `0` - line start absolute byte index
///
/// `1` - spanned line components
type DetailedComponentizedLine = (
    ByteIdx,
    ComponentizedLine,
);

/// ## lifetimes
/// - `lxr_src`: the lifetime of the borrowed string that is being lexed
pub struct MiniYamlLexer<'lxr_src> {
    _src: &'lxr_src str,
}

const KEY_SEP_CHAR: char = ':';
const COMMENT_START_CHAR: char = '#';

impl<'lxr_src> MiniYamlLexer<'lxr_src> {
    pub fn new_from_str(miniyaml_document: &'lxr_src str) -> Self {
        Self {
            _src: miniyaml_document,
        }
    }

    pub fn lex(self) -> Vec<DetailedComponentizedLine> {
        let lines_and_raw_spanned_lines_tup2 =
            Self::_split_into_lines_retaining_line_terms(self._src)
            .into_iter()
            .map(|raw_spanned_line| {
                let line = {
                    let start_abs_idx = (raw_spanned_line.raw.0).0;
                    let end_abs_idx = {
                        // end at the beginning of the term, if present
                        let opt_term_start_abs_idx = raw_spanned_line.term
                            .map(|term| (term.0).0);
                        // otherwise use the end of the 'raw' span, which is the
                        // end of the line
                        let raw_end_abs_idx = (raw_spanned_line.raw.1).0;

                        opt_term_start_abs_idx.unwrap_or(raw_end_abs_idx)
                    };

                    &self._src[start_abs_idx..end_abs_idx]
                };

                (line, raw_spanned_line)
            })
            .collect::<Vec<_>>();

        let detailed_comp_lines = lines_and_raw_spanned_lines_tup2.into_iter()
            .map(|(line, raw_spanned_line)| {
                let line_start_abs_bx = raw_spanned_line.raw.0;
                let comp_line = self._componentize_line(raw_spanned_line, line);
                (line_start_abs_bx, comp_line)
            })
            .collect::<Vec<_>>();

        detailed_comp_lines
    }

    fn _split_into_lines_retaining_line_terms(doc: &str) -> Vec<IntermediateRawSpannedLine> {
        let mut iter = doc.char_indices().peekable();

        let mut ret = vec![];
        let mut line_start_abs_bx = ByteIdx(0 /* start at beginning of document */);

        while let Some((ch_start_idx, ch)) = iter.next() {
            let ch_start_abs_bx = ByteIdx(ch_start_idx);
            let ch_end_abs_bx = ByteIdx(ch_start_idx + ch.len_utf8());

            let opt_next_tup2 = iter.peek()
                // clone to avoid subsequent borrow troubles
                .map(|tup2| tup2.clone());

            let opt_line_term_span_tup2 = match ch {
                '\n' => Some((ch_start_abs_bx.clone(), ch_end_abs_bx.clone())),
                '\r' => match opt_next_tup2 {
                    Some((lf_start_abs_idx, '\n')) => {
                        // advance over the `\n`
                        iter.next();

                        // high-exclusive
                        let end_abs_idx = lf_start_abs_idx + '\n'.len_utf8();

                        Some((
                            ch_start_abs_bx,
                            end_abs_idx.into(),
                        ))
                    },
                    _ => Some((ch_start_abs_bx, ch_end_abs_bx)),
                },
                _ => None,
            };

            match (opt_line_term_span_tup2, opt_next_tup2) {
                (Some(line_term_span), Some(_)) => {
                    ret.push(IntermediateRawSpannedLine {
                        raw: ByteIdxSpan(line_start_abs_bx, line_term_span.1.clone()),
                        term: Some(line_term_span.into()),
                    });

                    line_start_abs_bx = line_term_span.1;
                },
                (Some(line_term_span), None) => {
                    ret.push(IntermediateRawSpannedLine {
                        raw: ByteIdxSpan(line_start_abs_bx, line_term_span.1),
                        term: Some(line_term_span.into()),
                    });
                },
                (None, None) => {
                    ret.push(IntermediateRawSpannedLine {
                        raw: ByteIdxSpan(line_start_abs_bx, ch_end_abs_bx),
                        term: None,
                    });
                },
                (None, Some(_)) => { /* most likely case */ },
            };
        }

        ret
    }

    /// ## arguments
    /// `line`: if componentizing `hello\n`, this would be `hello`
    fn _componentize_line(
        &self,
        raw_spanned_line: IntermediateRawSpannedLine,
        line: &'lxr_src str,
    ) -> ComponentizedLine {
        let mut ret = ComponentizedLine {
            indent: Default::default(),
            key: Default::default(),
            key_sep: Default::default(),
            value: Default::default(),
            comment: Default::default(),
            term: raw_spanned_line.term,
        };

        let is_line_empty = line.is_empty();
        if is_line_empty {
            return ret;
        }

        let line_start_abs_bx = raw_spanned_line.raw.0;
        let line_end_abs_bx = raw_spanned_line.line_end();

        let is_line_whitespace_only = !is_line_empty && line.trim().is_empty();
        if is_line_whitespace_only {
            let indent_end_abs_bx = raw_spanned_line.term.map(|term_span| term_span.0).unwrap_or(line_end_abs_bx);
            let opt_indent_span = ByteIdxSpan(line_start_abs_bx, indent_end_abs_bx).into();
            ret.indent = opt_indent_span;
            return ret;
        }

        let first_non_whitespace_rel_idx = line.find(|ch: char| !ch.is_whitespace())
            .unwrap(/* earlier empty/whitespace-only returns make this safe */);

        let first_non_whitespace_abs_bx = (
            line_start_abs_bx.0 + first_non_whitespace_rel_idx
        ).into();

        ret.indent = if first_non_whitespace_rel_idx == 0 {
            None
        } else {
            ByteIdxSpan(line_start_abs_bx, first_non_whitespace_abs_bx).into()
        };

        // TODO: escaped_comment_with_actual_comment_after test
        let opt_first_comment_start_rel_idx = line.find(COMMENT_START_CHAR);
        let opt_comment_span = opt_first_comment_start_rel_idx
            .and_then(|comment_start_rel_idx| {
                if comment_start_rel_idx == first_non_whitespace_rel_idx {
                    // entire line from this point onwards is a comment
                    return ByteIdxSpan(first_non_whitespace_abs_bx, line_end_abs_bx).into();
                }

                let opt_text_before_comment_start = line.get(comment_start_rel_idx-1..comment_start_rel_idx);
                let is_escaped_comment_start = opt_text_before_comment_start
                    .map(|txt| txt.starts_with('\\'))
                    .unwrap_or(false);

                if is_escaped_comment_start {
                    return None;
                }

                let start_abs_bx = (line_start_abs_bx.0 + comment_start_rel_idx).into();
                ByteIdxSpan(start_abs_bx, line_end_abs_bx).into()
        });

        ret.comment = opt_comment_span;

        let opt_key_sep_start_rel_idx = line.find(KEY_SEP_CHAR);
        let opt_key_sep_span = opt_key_sep_start_rel_idx.and_then(|rel_idx| {
            let start_abs_bx = ByteIdx(line_start_abs_bx.0 + rel_idx);
            let end_abs_bx = (start_abs_bx.0 + KEY_SEP_CHAR.len_utf8()).into();
            ByteIdxSpan(start_abs_bx, end_abs_bx).into()
        });

        ret.key_sep = opt_key_sep_span;

        let opt_key_span: Option<ByteIdxSpan> =
            if ret.comment.map(|span| span.0) == Some(first_non_whitespace_abs_bx) {
                // line is either comment-only or indentation then comment
                None
            } else {
                match (opt_key_sep_span, opt_comment_span) {
                    (Some(key_sep_span), Some(comment_span)) => {
                        let key_end_abs_idx = ((key_sep_span.0).0).min((comment_span.0).0);

                        ByteIdxSpan(
                            first_non_whitespace_abs_bx,
                            key_end_abs_idx.into(),
                        )
                    },
                    (Some(key_sep_span), None) => ByteIdxSpan(
                        first_non_whitespace_abs_bx,
                        key_sep_span.0,
                    ),
                    (None, Some(comment_span)) => ByteIdxSpan(
                        first_non_whitespace_abs_bx,
                        comment_span.0,
                    ),
                    (None, None) => ByteIdxSpan(
                        first_non_whitespace_abs_bx,
                        line_end_abs_bx,
                    ),
                }.into()
            };

        ret.key = opt_key_span;

        // the 'value' portion exists between key-sep and either the comment
        // or end-of-line if there is no comment
        let opt_value_span: Option<ByteIdxSpan> = opt_key_sep_span.and_then(|key_sep_span| {
            let key_sep_end_rel_idx = (key_sep_span.1).0 - line_start_abs_bx.0;
            let line_after_key_sep = &line[key_sep_end_rel_idx..];

            let opt_value_start_rel_idx = line_after_key_sep.find(|ch: char| !ch.is_whitespace())
                .map(|rel_rel_idx| rel_rel_idx + key_sep_end_rel_idx);

            match (opt_value_start_rel_idx, opt_comment_span) {
                (Some(value_start_rel_idx), Some(comment_span)) => {
                    let line_start_abs_idx = line_start_abs_bx.0;
                    let comment_start_abs_bx = comment_span.0;
                    let comment_start_abs_idx = comment_start_abs_bx.0;
                    let value_start_abs_idx = line_start_abs_idx + value_start_rel_idx;
                    let comment_start_rel_idx = comment_start_abs_idx - line_start_abs_idx;

                    if value_start_rel_idx < comment_start_rel_idx {
                        ByteIdxSpan(
                            value_start_abs_idx.into(),
                            comment_start_abs_bx,
                        ).into()
                    } else {
                        None
                    }
                },
                (Some(value_start_rel_idx), None) => ByteIdxSpan(
                    (line_start_abs_bx.0 + value_start_rel_idx).into(),
                    line_end_abs_bx,
                ).into(),
                _ => None,
            }
        });

        ret.value = opt_value_span;

        ret
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    const NEWLINE_LF: &str = "\n";

    fn _conv_raw_spanned_line_to_subslice_incl_term(
        src: &str,
        IntermediateRawSpannedLine { raw: ByteIdxSpan(ByteIdx(start), ByteIdx(end)), .. }: IntermediateRawSpannedLine,
    ) -> &str {
        &src[start..end]
    }

    #[test]
    fn raw_spanned_line_calc_line_end_does_not_include_term_if_term_exists() {
        // arrange
        let raw_spanned_line = IntermediateRawSpannedLine {
            raw: ByteIdxSpan(0.into(), 5.into()),
            term: Some(ByteIdxSpan(4.into(), 5.into())),
        };

        let expected_line_end_bx = ByteIdx(4);

        // act
        let actual_line_end_bx = raw_spanned_line.line_end();

        // assert
        assert_eq!(
            expected_line_end_bx,
            actual_line_end_bx,
        );
    }

    #[test]
    fn raw_spanned_line_calc_line_end_at_end_of_raw_if_term_does_not_exist() {
        // arrange
        let raw_spanned_line = IntermediateRawSpannedLine {
            raw: ByteIdxSpan(0.into(), 5.into()),
            term: None,
        };

        let expected_line_end_bx = ByteIdx(5);

        // act
        let actual_line_end_bx = raw_spanned_line.line_end();

        // assert
        assert_eq!(
            expected_line_end_bx,
            actual_line_end_bx,
        );
    }

    #[test]
    fn determines_raw_spanned_line_with_lf_term() {
        // arrange
        let input = "hello, world\n";
        let expected_raw_spanned_lines = vec![ IntermediateRawSpannedLine {
            raw: (0, 13).into(),
            term: Some((12, 13).into()),
        } ];

        // act
        let actual_raw_spanned_lines = MiniYamlLexer::_split_into_lines_retaining_line_terms(input);

        // assert
        assert_eq!(
            expected_raw_spanned_lines,
            actual_raw_spanned_lines,
        );
    }

    #[test]
    fn determines_raw_spanned_line_with_crlf_term() {
        // arrange
        let input = "welcome\r\n";
        let expected_raw_spanned_lines = vec![ IntermediateRawSpannedLine {
            raw: (0, 9).into(),
            term: Some((7, 9).into()),
        } ];

        // act
        let actual_raw_spanned_lines = MiniYamlLexer::_split_into_lines_retaining_line_terms(input);

        // assert
        assert_eq!(
            expected_raw_spanned_lines,
            actual_raw_spanned_lines,
        );
    }

    #[test]
    fn determines_raw_spanned_line_without_term() {
        // arrange
        let input = "hoopla";
        let expected_raw_spanned_lines = vec![ IntermediateRawSpannedLine {
            raw: (0, 6).into(),
            term: None,
        } ];

        // act
        let actual_raw_spanned_lines = MiniYamlLexer::_split_into_lines_retaining_line_terms(input);

        // assert
        assert_eq!(
            expected_raw_spanned_lines,
            actual_raw_spanned_lines,
        );
    }

    #[test]
    fn determines_raw_spanned_lines_with_cr_and_lf_terms() {
        // arrange
        let input = "h\rello\n";
        let expected_raw_spanned_lines = vec![
            IntermediateRawSpannedLine {
                raw: (0, 2).into(),
                term: Some((1, 2).into()),
            },
            IntermediateRawSpannedLine {
                raw: (2, 7).into(),
                term: Some((6, 7).into()),
            },
        ];

        // act
        let actual_raw_spanned_lines = MiniYamlLexer::_split_into_lines_retaining_line_terms(input);

        // assert
        assert_eq!(
            expected_raw_spanned_lines,
            actual_raw_spanned_lines,
        );
    }

    #[test]
    fn _split_doc_into_lines_given_empty_str_returns_empty_collection() {
        // arrange
        let input_doc = "";

        // act
        let actual_raw_spanned_lines = MiniYamlLexer::_split_into_lines_retaining_line_terms(input_doc);

        // assert
        assert!(actual_raw_spanned_lines.is_empty());
    }

    #[test]
    fn _split_doc_into_lines() {
        // arrange
        let raw_doc_lines = vec![
            "\n",
            "foo\n",
            "\r\n",
            "qux:\n",
            "    baz\r",
            "zoop\n",
        ];

        let doc = &raw_doc_lines.join("");

        // act
        let actual_raw_spanned_lines = MiniYamlLexer::_split_into_lines_retaining_line_terms(doc);
        let actual_lines = actual_raw_spanned_lines.into_iter()
            .map(|raw_spanned_line| _conv_raw_spanned_line_to_subslice_incl_term(doc, raw_spanned_line))
            .collect::<Vec<_>>();

        // assert
        assert_eq!(
            raw_doc_lines,
            actual_lines
        );
    }

    #[test]
    fn whitespace_only_lines_componentize_with_correct_start_and_end() {
        // arrange
        let input_miniyaml_doc = vec![
            "	 \n",
            " 	\n",
            "	\r\n",
            " \r",
        ].join("");

        let expected_indent_spans = [
            Some((0, 2)),
            Some((3, 5)),
            Some((6, 7)),
            Some((9, 10)),
        ].iter()
        .map(|opt_span_tup2| opt_span_tup2.and_then(|(start, end)| ByteIdxSpan(start.into(), end.into()).into()))
        .collect::<Vec<_>>();

        // act
        let detailed_comp_lines = MiniYamlLexer::new_from_str(&input_miniyaml_doc).lex();
        let actual_indent_spans = detailed_comp_lines.into_iter()
            .map(|(_, comp_line)| comp_line.indent)
            .collect::<Vec<_>>();

        // assert
        assert_eq!(
            expected_indent_spans,
            actual_indent_spans,
        );
    }

    /* TODO: makes diagnostic reporting easier (can easily calculate column number from `raw`)
    #[test]
    fn comp_line_contains_raw() {
        let _ = ComponentizedLine {
            raw: ByteIdxSpan(ByteIdx(0), ByteIdx(15)),
            indent: None,
            key: None,
            key_sep: None,
            value: None,
            comment: None,
            term: None,
        };
    }
    */

    #[test]
    fn lexer_given_doc_with_only_lf_returns_single_comp_line_with_term_set() {
        // arrange
        let input_miniyaml_doc = vec![
            "",
        ].join(NEWLINE_LF) + NEWLINE_LF;

        let expected_comp_lines = vec![
            ComponentizedLine {
                indent: None,
                key: None,
                key_sep: None,
                value: None,
                comment: None,
                term: Some(ByteIdxSpan(ByteIdx(0), ByteIdx(1))),
            },
        ];

        // act
        let detailed_comp_lines = MiniYamlLexer::new_from_str(&input_miniyaml_doc).lex();
        let actual_comp_lines = detailed_comp_lines.into_iter()
            .map(|(_, comp_line)| comp_line)
            .collect::<Vec<_>>();

        // assert
        assert_eq!(
            expected_comp_lines,
            actual_comp_lines,
        );
    }

    #[test]
    fn lexer_given_doc_with_indent_then_lf_returns_single_comp_line_with_indent_and_term_set() {
        // arrange
        let input_miniyaml_doc = vec![
            " ",
        ].join(NEWLINE_LF) + NEWLINE_LF;

        let expected_comp_lines = vec![
            ComponentizedLine {
                indent: Some(ByteIdxSpan(ByteIdx(0), ByteIdx(1))),
                key: None,
                key_sep: None,
                value: None,
                comment: None,
                term: Some(ByteIdxSpan(ByteIdx(1), ByteIdx(2))),
            },
        ];

        // act
        let detailed_comp_lines = MiniYamlLexer::new_from_str(&input_miniyaml_doc).lex();
        let actual_comp_lines = detailed_comp_lines.into_iter()
            .map(|(_, comp_line)| comp_line)
            .collect::<Vec<_>>();

        // assert
        assert_eq!(
            expected_comp_lines,
            actual_comp_lines,
        );
    }

    #[test]
    fn lexer_given_doc_with_indent_then_key_text_then_lf_returns_single_comp_line_with_indent_and_key_and_term_set() {
        // arrange
        let input_miniyaml_doc = vec![
            " x",
        ].join(NEWLINE_LF) + NEWLINE_LF;

        let expected_comp_lines = vec![
            ComponentizedLine {
                indent: Some(ByteIdxSpan(ByteIdx(0), ByteIdx(1))),
                key: Some(ByteIdxSpan(ByteIdx(1), ByteIdx(2))),
                key_sep: None,
                value: None,
                comment: None,
                term: Some(ByteIdxSpan(ByteIdx(2), ByteIdx(3))),
            }
        ];

        // act
        let detailed_comp_lines = MiniYamlLexer::new_from_str(&input_miniyaml_doc).lex();
        let actual_comp_lines = detailed_comp_lines.into_iter()
            .map(|(_, comp_line)| comp_line)
            .collect::<Vec<_>>();

        // assert
        assert_eq!(
            expected_comp_lines,
            actual_comp_lines,
        );
    }

    #[test]
    fn lexer_given_doc_with_key_text_then_lf_returns_single_comp_line_with_key_and_term_set() {
        // arrange
        let input_miniyaml_doc = vec![
            "x",
        ].join(NEWLINE_LF) + NEWLINE_LF;

        let expected_comp_lines = vec![
            ComponentizedLine {
                indent: None,
                key: Some(ByteIdxSpan(ByteIdx(0), ByteIdx(1))),
                key_sep: None,
                value: None,
                comment: None,
                term: Some(ByteIdxSpan(ByteIdx(1), ByteIdx(2))),
            }
        ];

        // act
        let detailed_comp_lines = MiniYamlLexer::new_from_str(&input_miniyaml_doc).lex();
        let actual_comp_lines = detailed_comp_lines.into_iter()
            .map(|(_, comp_line)| comp_line)
            .collect::<Vec<_>>();

        // assert
        assert_eq!(
            expected_comp_lines,
            actual_comp_lines,
        );
    }

    #[test]
    fn lexer_given_doc_with_indent_then_comment_then_lf_returns_single_comp_line_with_indent_and_comment_and_term_set() {
        // arrange
        let input_miniyaml_doc = vec![
            " # hello",
        ].join(NEWLINE_LF) + NEWLINE_LF;

        let expected_comp_lines = vec![
            ComponentizedLine {
                indent: Some(ByteIdxSpan(ByteIdx(0), ByteIdx(1))),
                key: None,
                key_sep: None,
                value: None,
                comment: Some(ByteIdxSpan(ByteIdx(1), ByteIdx(8))),
                term: Some(ByteIdxSpan(ByteIdx(8), ByteIdx(9))),
            }
        ];

        // act
        let detailed_comp_lines = MiniYamlLexer::new_from_str(&input_miniyaml_doc).lex();
        let actual_comp_lines = detailed_comp_lines.into_iter()
            .map(|(_, comp_line)| comp_line)
            .collect::<Vec<_>>();

        // assert
        assert_eq!(
            expected_comp_lines,
            actual_comp_lines,
        );
    }

    #[test]
    #[ignore = "test key_sep_after_comment_start has higher priority"]
    fn escaped_comment_with_actual_comment_after() {
        // arrange
        let input_miniyaml_doc = vec![
            "\\# escaped # actual",
        ].join(NEWLINE_LF) + NEWLINE_LF;

        let expected_comp_lines = vec![
            ComponentizedLine {
                indent: None,
                key: Some(ByteIdxSpan(ByteIdx(0), ByteIdx(11))),
                key_sep: None,
                value: None,
                comment: Some(ByteIdxSpan(ByteIdx(11), ByteIdx(19))),
                term: Some(ByteIdxSpan(ByteIdx(19), ByteIdx(20))),
            }
        ];

        // act
        let detailed_comp_lines = MiniYamlLexer::new_from_str(&input_miniyaml_doc).lex();
        let actual_comp_lines = detailed_comp_lines.into_iter()
            .map(|(_, comp_line)| comp_line)
            .collect::<Vec<_>>();

        // assert
        assert_eq!(
            expected_comp_lines,
            actual_comp_lines,
        );
    }

    #[test]
    #[ignore = "until this is encountered in the wild, or I feel like fixing it"]
    fn key_sep_after_comment_start() {
        // arrange
        let input_miniyaml_doc = vec![
            " # hello : world",
        ].join(NEWLINE_LF) + NEWLINE_LF;

        let expected_comp_lines = vec![
            ComponentizedLine {
                indent: Some(ByteIdxSpan(ByteIdx(0), ByteIdx(1))),
                key: None,
                key_sep: None,
                value: None,
                comment: Some(ByteIdxSpan(ByteIdx(1), ByteIdx(16))),
                term: Some(ByteIdxSpan(ByteIdx(16), ByteIdx(17))),
            }
        ];

        // act
        let detailed_comp_lines = MiniYamlLexer::new_from_str(&input_miniyaml_doc).lex();
        let actual_comp_lines = detailed_comp_lines.into_iter()
            .map(|(_, comp_line)| comp_line)
            .collect::<Vec<_>>();

        // assert
        assert_eq!(
            expected_comp_lines,
            actual_comp_lines,
        );
    }

    #[test]
    fn lexer_given_doc_with_key_then_key_sep_then_escaped_comment_returns_single_comp_line_with_key_and_key_sep_and_value_set() {
        // arrange
        let input_miniyaml_doc = vec![
            "hi:\\# foo",
        ].join(NEWLINE_LF) + NEWLINE_LF;

        let expected_comp_lines = vec![
            ComponentizedLine {
                indent: None,
                key: Some(ByteIdxSpan(ByteIdx(0), ByteIdx(2))),
                key_sep: Some(ByteIdxSpan(ByteIdx(2), ByteIdx(3))),
                value: Some(ByteIdxSpan(ByteIdx(3), ByteIdx(9))),
                comment: None,
                term: Some(ByteIdxSpan(ByteIdx(9), ByteIdx(10))),
            }
        ];

        // act
        let detailed_comp_lines = MiniYamlLexer::new_from_str(&input_miniyaml_doc).lex();
        let actual_comp_lines = detailed_comp_lines.into_iter()
            .map(|(_, comp_line)| comp_line)
            .collect::<Vec<_>>();

        // assert
        assert_eq!(
            expected_comp_lines,
            actual_comp_lines,
        );
    }

    #[test]
    fn lexer_given_doc_with_indent_then_key_then_comment_then_lf_returns_single_comp_line_with_indent_and_key_and_comment_and_term_set() {
        // arrange
        let input_miniyaml_doc = vec![
            " hi # hello",
        ].join(NEWLINE_LF) + NEWLINE_LF;

        let expected_comp_lines = vec![
            ComponentizedLine {
                indent: Some(ByteIdxSpan(ByteIdx(0), ByteIdx(1))),
                key: Some(ByteIdxSpan(ByteIdx(1), ByteIdx(4))),
                key_sep: None,
                value: None,
                comment: Some(ByteIdxSpan(ByteIdx(4), ByteIdx(11))),
                term: Some(ByteIdxSpan(ByteIdx(11), ByteIdx(12))),
            }
        ];

        // act
        let detailed_comp_lines = MiniYamlLexer::new_from_str(&input_miniyaml_doc).lex();
        let actual_comp_lines = detailed_comp_lines.into_iter()
            .map(|(_, comp_line)| comp_line)
            .collect::<Vec<_>>();

        // assert
        assert_eq!(
            expected_comp_lines,
            actual_comp_lines,
        );
    }

    #[test]
    fn lexer_given_doc_with_key_then_key_sep_then_lf_returns_single_comp_line_with_key_and_key_sep_and_term_set() {
        // arrange
        let input_miniyaml_doc = vec![
            "Packages:",
        ].join(NEWLINE_LF) + NEWLINE_LF;

        let expected_comp_lines = vec![
            ComponentizedLine {
                indent: None,
                key: Some(ByteIdxSpan(ByteIdx(0), ByteIdx(8))),
                key_sep: Some(ByteIdxSpan(ByteIdx(8), ByteIdx(9))),
                value: None,
                comment: None,
                term: Some(ByteIdxSpan(ByteIdx(9), ByteIdx(10))),
            }
        ];

        // act
        let detailed_comp_lines = MiniYamlLexer::new_from_str(&input_miniyaml_doc).lex();
        let actual_comp_lines = detailed_comp_lines.into_iter()
            .map(|(_, comp_line)| comp_line)
            .collect::<Vec<_>>();

        // assert
        assert_eq!(
            expected_comp_lines,
            actual_comp_lines,
        );
    }

    #[test]
    fn lexer_given_doc_with_key_then_key_sep_then_value_then_lf_returns_single_comp_line_with_key_and_key_sep_and_value_and_term_set() {
        // arrange
        let input_miniyaml_doc = vec![
            "x:y",
        ].join(NEWLINE_LF) + NEWLINE_LF;

        let expected_comp_lines = vec![
            ComponentizedLine {
                indent: None,
                key: Some(ByteIdxSpan(ByteIdx(0), ByteIdx(1))),
                key_sep: Some(ByteIdxSpan(ByteIdx(1), ByteIdx(2))),
                value: Some(ByteIdxSpan(ByteIdx(2), ByteIdx(3))),
                comment: None,
                term: Some(ByteIdxSpan(ByteIdx(3), ByteIdx(4))),
            }
        ];

        // act
        let detailed_comp_lines = MiniYamlLexer::new_from_str(&input_miniyaml_doc).lex();
        let actual_comp_lines = detailed_comp_lines.into_iter()
            .map(|(_, comp_line)| comp_line)
            .collect::<Vec<_>>();

        // assert
        assert_eq!(
            expected_comp_lines,
            actual_comp_lines,
        );
    }

    #[test]
    fn lexer_given_indent_then_key_then_key_sep_then_value_then_comment_then_lf_returns_single_comp_line_with_indent_and_key_and_key_sep_and_value_and_comment_and_term_set() {
        // arrange
        let input_miniyaml_doc = vec![
            " x:y#z",
        ].join(NEWLINE_LF) + NEWLINE_LF;

        let expected_comp_lines = vec![
            ComponentizedLine {
                indent: Some(ByteIdxSpan(ByteIdx(0), ByteIdx(1))),
                key: Some(ByteIdxSpan(ByteIdx(1), ByteIdx(2))),
                key_sep: Some(ByteIdxSpan(ByteIdx(2), ByteIdx(3))),
                value: Some(ByteIdxSpan(ByteIdx(3), ByteIdx(4))),
                comment: Some(ByteIdxSpan(ByteIdx(4), ByteIdx(6))),
                term: Some(ByteIdxSpan(ByteIdx(6), ByteIdx(7))),
            }
        ];

        // act
        let detailed_comp_lines = MiniYamlLexer::new_from_str(&input_miniyaml_doc).lex();
        let actual_comp_lines = detailed_comp_lines.into_iter()
            .map(|(_, comp_line)| comp_line)
            .collect::<Vec<_>>();

        // assert
        assert_eq!(
            expected_comp_lines,
            actual_comp_lines,
        );
    }

    #[test]
    // #[ignore = "todo prior: tests for individual pieces"]
    fn _multiple_comp_lines() {
        // arrange
        let input_miniyaml_doc = vec![
            /* 1 */ "Packages:",
            /* 2 */ "    ~^Content/ts",
            /* 3 */ "    .",
            /* 4 */ "    $ts: ts",
            /* 5 */ "    ./mods/common: common",
            /* 6 */ "",
            /* 7 */ "non_ascii: 请务必取代",
        ].join(NEWLINE_LF) + NEWLINE_LF;

        let expected_comp_lines: Vec<ComponentizedLine> = vec![
            ComponentizedLine {
                indent: None,
                key: Some(ByteIdxSpan(ByteIdx(0), ByteIdx(8))),
                key_sep: Some(ByteIdxSpan(ByteIdx(8), ByteIdx(9))),
                value: None,
                comment: None,
                term: Some(ByteIdxSpan(ByteIdx(9), ByteIdx(10)))
            },
            ComponentizedLine {
                indent: Some(ByteIdxSpan(ByteIdx(10), ByteIdx(14))),
                key: Some(ByteIdxSpan(ByteIdx(14), ByteIdx(26))),
                key_sep: None,
                value: None,
                comment: None,
                term: Some(ByteIdxSpan(ByteIdx(26), ByteIdx(27))),
            },
            ComponentizedLine {
                indent: Some(ByteIdxSpan(ByteIdx(27), ByteIdx(31))),
                key: Some(ByteIdxSpan(ByteIdx(31), ByteIdx(32))),
                key_sep: None,
                value: None,
                comment: None,
                term: Some(ByteIdxSpan(ByteIdx(32), ByteIdx(33))),
            },
            ComponentizedLine {
                indent: Some(ByteIdxSpan(ByteIdx(33), ByteIdx(37))),
                key: Some(ByteIdxSpan(ByteIdx(37), ByteIdx(40))),
                key_sep: Some(ByteIdxSpan(ByteIdx(40), ByteIdx(41))),
                value: Some(ByteIdxSpan(ByteIdx(42), ByteIdx(44))),
                comment: None,
                term: Some(ByteIdxSpan(ByteIdx(44), ByteIdx(45))),
            },
            ComponentizedLine {
                indent: Some(ByteIdxSpan(ByteIdx(45), ByteIdx(49))),
                key: Some(ByteIdxSpan(ByteIdx(49), ByteIdx(62))),
                key_sep: Some(ByteIdxSpan(ByteIdx(62), ByteIdx(63))),
                value: Some(ByteIdxSpan(ByteIdx(64), ByteIdx(70))),
                comment: None,
                term: Some(ByteIdxSpan(ByteIdx(70), ByteIdx(71))),
            },
            ComponentizedLine {
                indent: None,
                key: None,
                key_sep: None,
                value: None,
                comment: None,
                term: Some(ByteIdxSpan(ByteIdx(71), ByteIdx(72))),
            },
            ComponentizedLine {
                indent: None,
                key: Some(ByteIdxSpan(ByteIdx(72), ByteIdx(81))),
                key_sep: Some(ByteIdxSpan(ByteIdx(81), ByteIdx(82))),
                value: Some(ByteIdxSpan(ByteIdx(83), ByteIdx(98))),
                comment: None,
                term: Some(ByteIdxSpan(ByteIdx(98), ByteIdx(99))),
            },
        ];

        // act
        let detailed_comp_lines = MiniYamlLexer::new_from_str(&input_miniyaml_doc).lex();
        let actual_comp_lines = detailed_comp_lines.into_iter()
            .map(|(_, comp_line)| comp_line)
            .collect::<Vec<_>>();

        // assert
        assert_eq!(
            expected_comp_lines,
            actual_comp_lines,
        );
    }
}
