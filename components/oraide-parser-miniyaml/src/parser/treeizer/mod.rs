// This file is part of oraide.  See <https://github.com/Phrohdoh/oraide>.
// 
// oraide is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License version 3
// as published by the Free Software Foundation.
// 
// oraide is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
// 
// You should have received a copy of the GNU Affero General Public License
// along with oraide.  If not, see <https://www.gnu.org/licenses/>.

//! # `treeizer`
//!
//! Transform a collection of `Node`s into a `Tree`
//!
//! ---
//!
//! The entrypoint to this module is the `Treeizer` struct.
//!

pub use indextree::{
    NodeId as ArenaNodeId,
};

use crate::{
    Node,
};

pub type Arena = indextree::Arena<Node>;

// https://github.com/OpenRA/OpenRA/blob/30103da2db58b8fba09b45b6d9dfbb7049a2c449/OpenRA.Game/MiniYaml.cs#L93
const SPACES_PER_INDENT_LEVEL: usize = 4;

// Just to be consistent in the code (so we don't have `1`s scattered about).
const TABS_PER_INDENT_LEVEL: usize = 1;

/// A [`Tree`] groups an [`indextree::Arena`] with all of its [`indextree::NodeId`]s
///
/// [`Tree`]: struct.Tree.html
/// [`indextree::Arena`]: ../indextree/struct.Arena.html
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tree {
    /// All IDs for nodes that exist in `arena` with the first item always
    /// being the sentinel for parent-less nodes
    pub node_ids: Vec<ArenaNodeId>,

    /// The `indextree::Arena` that contains `Node`s
    pub arena: Arena,
}

impl Tree {
    pub fn from(node_ids: Vec<ArenaNodeId>, arena: Arena) -> Self {
        Self {
            node_ids,
            arena,
        }
    }
}

/// Used to store/calculate indentation level delta between two *thing*s
///
/// In the context of textual MiniYaml this is used to
/// determine the relationship, if any, between two nodes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndentLevelDelta {
    /// Something has a lower indentation level than something else
    LessIndented(usize),

    /// Two things have the same indentation level
    NoChange,

    /// Something has a higher indentation level than something else
    MoreIndented(usize),
}

impl IndentLevelDelta {
    /// This is [`IndentLevelDelta::calc`] specialized for [`Node`]s
    ///
    /// ```rust
    /// # use oraide_span::{FileId};
    /// # use oraide_parser_miniyaml::{Tokenizer,Nodeizer,IndentLevelDelta};
    /// # let file_id = FileId(0);
    /// let src = "Hello:\n\tFoo:\n";
    /// # let mut tokenizer = Tokenizer::new(file_id, src);
    /// # let tokens = tokenizer.run();
    /// # let mut nodeizer = Nodeizer::new(tokens.into_iter());
    /// # let nodes = nodeizer.run();
    /// # assert_eq!(nodes.len(), 2);
    /// // ... get `nodes: Vec<Node>` from `src` ...
    /// let a = nodes.get(0).unwrap();
    /// let b = nodes.get(1).unwrap();
    /// let delta = IndentLevelDelta::nodes(a, b);
    /// assert_eq!(delta, IndentLevelDelta::MoreIndented(1));
    /// ```
    ///
    /// [`Node`]: struct.Node.html
    /// [`IndentLevelDelta::calc`]: enum.IndentLevelDelta.html#method.calc
    pub fn nodes(a: &Node, b: &Node) -> Self {
        Self::calc(a, b, Node::indentation_level)
    }

    /// Calculate the indentation delta for two `T` instances using `fn_indent_level`
    /// to determine the indentation level for each `T`
    ///
    /// # Example
    ///
    /// ```rust
    /// # use oraide_parser_miniyaml::{IndentLevelDelta};
    /// #[derive(PartialEq, Eq, PartialOrd, Ord)]
    /// struct Thing {
    ///     leading_spaces: usize,
    /// }
    ///
    /// impl Thing {
    ///     fn indentation_level(&self) -> usize {
    ///         // 4 spaces per indentation level
    ///         self.leading_spaces / 4
    ///     }
    /// }
    ///
    /// let a = Thing { leading_spaces: 8 };
    /// let b = Thing { leading_spaces: 4 };
    ///
    /// let delta = IndentLevelDelta::calc(&a, &b, Thing::indentation_level);
    /// assert_eq!(delta, IndentLevelDelta::LessIndented(1));
    /// ```
    pub fn calc<T: Ord, F: Fn(&T) -> usize>(a: &T, b: &T, fn_indent_level: F) -> Self {
        let a_indent_level = fn_indent_level(a);
        let b_indent_level = fn_indent_level(b);

        use std::cmp::Ordering;

        match b_indent_level.cmp(&a_indent_level) {
            Ordering::Equal => IndentLevelDelta::NoChange,
            Ordering::Greater => IndentLevelDelta::MoreIndented(b_indent_level - a_indent_level),
            Ordering::Less => IndentLevelDelta::LessIndented(a_indent_level - b_indent_level),
        }
    }
}

/// Transform a collection of [`Node`]s into a [`Tree`]
///
/// # Lifetimes
/// `'text`: the underlying text that is being tokenized
///
/// # Type Parameters
/// `I`: An _iterable_ that yields [`Node`]s
///
/// # Example
/// ```rust
/// # use oraide_parser_miniyaml::{Node,Tree,Treeizer};
/// let nodes: Vec<Node> = vec![];
///
/// // Create a `Treeizer`
/// let mut treeizer = Treeizer::new(nodes.into_iter(), "your source text");
///
/// // Build a `Tree`
/// let tree: Tree = treeizer.run();
/// ```
///
/// [`Node`]: struct.Node.html
/// [`Tree`]: struct.Tree.html
pub struct Treeizer<'text, I: Iterator<Item = Node>> {
    /// The collection of `Node`s being transformed into a `Tree`
    nodes: I,

    /// The underlying text that is being treeized
    text: &'text str,
}

impl<'text, I: Iterator<Item = Node>> Treeizer<'text, I> {
    pub fn new(nodes: I, text: &'text str) -> Self {
        Self {
            nodes,
            text,
        }
    }

    /// Build a [`Tree`]
    ///
    /// [`Tree`]: struct.Tree.html
    pub fn run(&mut self) -> Tree {
        let mut arena = Arena::new();
        let parentless_sentinel_node_id = arena.new_node(Node::new_empty());

        let mut parent_node_ids = Vec::<ArenaNodeId>::new();
        let mut all_node_ids = vec![ parentless_sentinel_node_id ];

        while let Some(node) = self.nodes.next() {
            if node.is_whitespace_only() {
                // TODO:diag let span = node.span().expect(&format!(
                // TODO:diag     "`{}` *should* guarantee that the node has a non-`None` span",
                // TODO:diag     stringify!(Node::is_whitespace_only)
                // TODO:diag ));

                // TODO:diag self.add_diagnostic(
                // TODO:diag     Diagnostic::new_warning("Found a whitespace-only line")
                // TODO:diag         .with_code("A:W0001")
                // TODO:diag         .with_label(Label::new_primary(span))
                // TODO:diag );

                // TODO:diag self.add_diagnostic(Diagnostic::new_help("Consider making the line empty"));

                // We can't do anything intelligent with a whitespace-only
                // node so continue on to the next node.
                continue;
            }

            // if `node` is indented
            if let Some(shrd_node_indent_token) = node.indentation_token.as_ref() {
                let node_indent_slice = {
                    let start = shrd_node_indent_token.span.start().to_usize();
                    let end_exc = shrd_node_indent_token.span.end_exclusive().to_usize();
                    &self.text[start..end_exc]
                };

                let node_indent_slice_len = node_indent_slice.len();

                let is_all_space = node_indent_slice.chars().all(|ch| ch == ' ');
                let is_all_tab = node_indent_slice.chars().all(|ch| ch == '\t');

                if !is_all_space && !is_all_tab {
                    // TODO:diag self.add_diagnostic(
                    // TODO:diag     Diagnostic::new_error("Indentation must be entirely made up of either spaces or tabs, but not both")
                    // TODO:diag         .with_code("A:E0001")
                    // TODO:diag         .with_label(Label::new_primary(shrd_node_indent_token.span.clone()))
                    // TODO:diag );

                    let node_id = arena.new_node(node);
                    parentless_sentinel_node_id.append(node_id, &mut arena);

                    // Since the indentation is bogus there is no reason
                    // to attempt to determine the parent (it'd just be a waste of cycles
                    // for a very small chance at being correct), just continue on to
                    // the next node.
                    continue;
                } else if is_all_space && node_indent_slice_len % SPACES_PER_INDENT_LEVEL != 0 {
                    // TODO:diag self.add_diagnostic(
                    // TODO:diag     Diagnostic::new_error(format!(
                    // TODO:diag         "Column number must be a multiple of {} when using spaces",
                    // TODO:diag         SPACES_PER_INDENT_LEVEL
                    // TODO:diag     )).with_code("A:E0002")
                    // TODO:diag       .with_label(Label::new_primary(shrd_node_indent_token.span.clone()))
                    // TODO:diag );

                    // TODO:diag self.add_diagnostic(
                    // TODO:diag     Diagnostic::new_help(format!(
                    // TODO:diag         "Column number is currently {}",
                    // TODO:diag         node_indent_slice_len
                    // TODO:diag     ))
                    // TODO:diag );
                }

                match parent_node_ids.last() {
                    Some(last_parent_node_id) => {
                        let last_parent_node_id = *last_parent_node_id;

                        let shrd_last_parent_node = match arena.get(last_parent_node_id) {
                            Some(n) => n,
                            None => {
                                let msg = format!(
                                    "Expected `{}` with id `{}` to be in `{}`",
                                    stringify!(ArenaNode),
                                    last_parent_node_id,
                                    stringify!(arena)
                                );

                                // TODO:diag self.add_diagnostic(
                                // TODO:diag     Diagnostic::new_bug(msg.clone())
                                // TODO:diag         .with_code("A:E0003")
                                // TODO:diag         .with_label(Label::new_primary(node.span().unwrap()))
                                // TODO:diag );

                                panic!("{}", msg)
                            },
                        };

                        match IndentLevelDelta::nodes(&shrd_last_parent_node.get(), &node) {
                            IndentLevelDelta::NoChange => {
                                let _sibling_id = parent_node_ids.pop(); // remove the sibling's ID
                                // TODO:diag let node_span_opt = node.span();
                                let node_id = arena.new_node(node);

                                match parent_node_ids.last() {
                                    Some(shrd_last_parent_node_id) => {
                                        if let Err(e) = shrd_last_parent_node_id.checked_append(node_id, &mut arena) {
                                            let err_msg = format!(
                                                "Got an error attempting to make `{}` a child of `{}`: {:?}",
                                                node_id,
                                                shrd_last_parent_node_id,
                                                e
                                            );

                                            log::error!("{}", err_msg);

                                            // TODO:diag let mut diag = Diagnostic::new_bug(err_msg).with_code("A:E0006");

                                            // TODO:diag if let Some(span) = node_span_opt {
                                            // TODO:diag     diag = diag.with_label(Label::new_primary(span));
                                            // TODO:diag }

                                            // TODO:diag self.add_diagnostic(diag);
                                        }
                                    },
                                    None => {
                                        let err_msg = format!(
                                            "Determined there was no indentation level change from the previous node, but no ID found in `{}`",
                                            stringify!(parent_node_ids),
                                        );

                                        log::error!("{}", err_msg);

                                        // TODO:diag let mut diag = Diagnostic::new_bug(err_msg).with_code("A:E0007");

                                        // TODO:diag if let Some(span) = node_span_opt {
                                        // TODO:diag     diag = diag.with_label(Label::new_primary(span));
                                        // TODO:diag }

                                        // TODO:diag self.add_diagnostic(diag);
                                    },
                                }

                                parent_node_ids.push(node_id);
                                all_node_ids.push(node_id);
                            },
                            IndentLevelDelta::MoreIndented(col_num_diff) => {
                                if is_all_space && col_num_diff != SPACES_PER_INDENT_LEVEL {
                                    // TODO:diag let mut diag = Diagnostic::new_error(format!(
                                    // TODO:diag     "Indentation difference must be {} spaces",
                                    // TODO:diag     SPACES_PER_INDENT_LEVEL
                                    // TODO:diag )).with_code("A:E0004");

                                    // TODO:diag if let Some(node_span) = node.indentation_token.as_ref().map(|token| token.span) {
                                    // TODO:diag     diag = diag.with_label(
                                    // TODO:diag         Label::new_primary(node_span)
                                    // TODO:diag     );
                                    // TODO:diag }

                                    // TODO:diag self.add_diagnostic(diag);

                                    // TODO:diag self.add_diagnostic(
                                    // TODO:diag     Diagnostic::new_help(format!(
                                    // TODO:diag         "Consider deleting {} space(s)",
                                    // TODO:diag         col_num_diff
                                    // TODO:diag     ))
                                    // TODO:diag );
                                } else if is_all_tab && col_num_diff != TABS_PER_INDENT_LEVEL {
                                    // TODO:diag self.add_diagnostic(
                                    // TODO:diag     Diagnostic::new_error(format!(
                                    // TODO:diag         "Indentation difference must be {} tab(s)",
                                    // TODO:diag         TABS_PER_INDENT_LEVEL
                                    // TODO:diag     )).with_code("A:E0005")
                                    // TODO:diag );
                                }

                                let node_id = arena.new_node(node);
                                parent_node_ids.push(node_id);
                                all_node_ids.push(node_id);

                                if let Err(e) = last_parent_node_id.checked_append(node_id, &mut arena) {
                                    log::error!(
                                        "Got an error attempting to make `{}` a child of `{}`: {:?}",
                                        node_id,
                                        last_parent_node_id,
                                        e
                                    );
                                }
                            },
                            IndentLevelDelta::LessIndented(_col_num_diff) => {
                                // Find the node that is 1 level unindented in reference to `node`
                                let parent_node_id_opt = {
                                    let desired_col_num_diff = if is_all_space {
                                        SPACES_PER_INDENT_LEVEL
                                    } else {
                                        TABS_PER_INDENT_LEVEL
                                    };

                                    let desired_indent_delta = IndentLevelDelta::LessIndented(desired_col_num_diff);

                                    let mut iter = parent_node_ids.iter().rev();
                                    iter.find(|&&id| {
                                        let shrd_iter_node = match arena.get(id) {
                                            Some(n) => n.get(),
                                            _ => return false,
                                        };

                                        let indent_delta = IndentLevelDelta::nodes(&node, shrd_iter_node);
                                        indent_delta == desired_indent_delta
                                    }).map(|id| *id)
                                };

                                if let Some(parent_node_id) = parent_node_id_opt {
                                    let parent_node_id_pos = parent_node_ids.iter().position(|&id| id == parent_node_id).unwrap();

                                    // The position we found earlier is 0-based but length is 1-based
                                    // so we must add 1 to the position when truncating.
                                    parent_node_ids.truncate(parent_node_id_pos + 1);

                                    // TODO:diag let node_span_opt = node.span();
                                    let node_id = arena.new_node(node);
                                    parent_node_ids.push(node_id);
                                    all_node_ids.push(node_id);

                                    if let Err(e) = parent_node_id.checked_append(node_id, &mut arena) {
                                        let err_msg = format!(
                                            "Got an error attempting to make `{}` a child of `{}`: {:?}",
                                            node_id,
                                            parent_node_id,
                                            e
                                        );

                                        log::error!("{}", err_msg);

                                        // TODO:diag let mut diag = Diagnostic::new_bug(err_msg).with_code("A:E0008");

                                        // TODO:diag if let Some(span) = node_span_opt {
                                        // TODO:diag     diag = diag.with_label(Label::new_primary(span));
                                        // TODO:diag }

                                        // TODO:diag self.add_diagnostic(diag);
                                    }
                                } else {
                                    // TODO:diag self.add_diagnostic(
                                    // TODO:diag     Diagnostic::new_error("Unable to determine parent node due to indentation")
                                    // TODO:diag         .with_code("A:E0009")
                                    // TODO:diag         .with_label(Label::new_primary(node.span().unwrap()))
                                    // TODO:diag );
                                }
                            },
                        }
                    },
                    None => {
                        // TODO:diag self.add_diagnostic(
                        // TODO:diag     Diagnostic::new_error("Unable to determine parent node due to indentation")
                        // TODO:diag         .with_code("A:E0010")
                        // TODO:diag         .with_label(Label::new_primary(node.span().unwrap()))
                        // TODO:diag );

                        let node_id = arena.new_node(node);
                        parent_node_ids.push(node_id);
                        all_node_ids.push(node_id);

                        parentless_sentinel_node_id.append(node_id, &mut arena);
                    },
                }
            } else {
                // The current `node` is not indented so it starts a new tree, meaning:
                // - it should not have a parent
                // - the previously-tracked IDs are no longer useful

                parent_node_ids.clear();

                let is_comment_only = node.is_comment_only();
                let node_id = arena.new_node(node);
                all_node_ids.push(node_id);

                // If this node is comment-only don't track its ID
                // as it should not be made a parent of another node.
                if !is_comment_only {
                    parent_node_ids.push(node_id);
                }
            }
        }

        /*
        for parentless_arena_node in parentless_sentinel_node_id.children(&arena).filter_map(|id| arena.get(id)) {
            let n = &parentless_arena_node.get();
            if let Some(span) = n.span() {
                self.add_diagnostic(
                    Diagnostic::new_error("Unable to determine parent for this node, please address any prior errors and try again")
                        .with_label(Label::new_primary(span))
                );
            }
        }
        */

        let tree = Tree::from(all_node_ids, arena);
        tree
    }
}