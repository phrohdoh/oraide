use language_reporting::{Diagnostic, Label};

use mltt_span::{FileSpan};

use indextree::{NodeId as ArenaNodeId};

use crate::{
    types::{
        Arena,
    },

    Node,
    Tree,
};

// https://github.com/OpenRA/OpenRA/blob/30103da2db58b8fba09b45b6d9dfbb7049a2c449/OpenRA.Game/MiniYaml.cs#L93
const SPACES_PER_INDENT_LEVEL: usize = 4;

// Just to be consistent in the code (so we don't have `1`s scattered about).
const TABS_PER_INDENT_LEVEL: usize = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IndentDelta {
    /// Node A is less indented than B
    LessIndented(usize),

    /// Node A and B have the same indentation level
    NoChange,

    /// Node A is more indented than B
    MoreIndented(usize),
}

pub struct Arborist<Nodes: Iterator> {
    /// The underlying stream of nodes
    nodes: Nodes,

    /// Diagnostics accumulated during tree-building
    diagnostics: Vec<Diagnostic<FileSpan>>,
}

/// # Returns
/// - `IndentDelta::NoChange` if `b`'s indentation level is equal to `a`'s
/// - `IndentDelta::MoreIndented(col_num_diff)` if `b`'s indentation level is greater than `a`'s
/// - `IndentDelta::LessIndented(col_num_diff)` if `b`'s indentation level is less than `a`'s
fn calc_indent_delta(a: &Node, b: &Node) -> IndentDelta {
    let a_indent_level = a.indentation_token.as_ref().map_or(0, |token| token.slice.len());
    let b_indent_level = b.indentation_token.as_ref().map_or(0, |token| token.slice.len());

    use std::cmp::Ordering;

    match b_indent_level.cmp(&a_indent_level) {
        Ordering::Equal => IndentDelta::NoChange,
        Ordering::Greater => IndentDelta::MoreIndented(b_indent_level - a_indent_level),
        Ordering::Less => IndentDelta::LessIndented(a_indent_level - b_indent_level),
    }
}

impl<'file, Nodes> Arborist<Nodes>
where
    Nodes: Iterator<Item = Node<'file>> + 'file,
{
    /// Create a new arborist from an iterator of `Node`s
    pub fn new(nodes: Nodes) -> Arborist<Nodes> {
        Self {
            nodes,
            diagnostics: vec![],
        }
    }

    /// Record a diagnostic
    fn add_diagnostic(&mut self, diagnostic: Diagnostic<FileSpan>) {
        if log::log_enabled!(log::Level::Debug) {
            let mut s = format!("diagnostic added ({})", diagnostic.severity.to_str());

            if let Some(label) = diagnostic.labels.first() {
                s.push_str(&format!(
                    " @ {}..{}",
                    label.span.start().to_usize(),
                    label.span.end().to_usize()
                ));
            }

            log::debug!("{}: {:?}", s, diagnostic.message);
        }

        self.diagnostics.push(diagnostic);
    }

    /// Take the diagnostics from the parser, leaving an empty collection
    pub fn take_diagnostics(&mut self) -> Vec<Diagnostic<FileSpan>> {
        std::mem::replace(&mut self.diagnostics, Vec::new())
    }

    /// Build a tree (backed by an `indextree::Arena`) of nodes
    /// 
    /// # Returns
    /// A `oraml::Tree` struct instance
    pub fn build_tree(&mut self) -> Tree<'file> {
        let mut arena = Arena::new();
        let parentless_sentinel_node_id = arena.new_node(Node::empty());

        let mut parent_node_ids = Vec::<ArenaNodeId>::new();
        let mut all_node_ids = vec![ parentless_sentinel_node_id ];

        while let Some(node) = self.nodes.next() {
            if node.is_whitespace_only() {
                let span = node.span().expect(&format!(
                    "`{}` *should* guarantee that the node has a non-`None` span",
                    stringify!(Node::is_whitespace_only)
                ));

                self.add_diagnostic(
                    Diagnostic::new_warning("Found a whitespace-only line")
                        .with_code("A:W0001")
                        .with_label(Label::new_primary(span))
                );

                self.add_diagnostic(Diagnostic::new_help("Consider making the line empty"));

                // We can't do anything intelligent with a whitespace-only
                // node so continue on to the next node.
                continue;
            }

            // if `node` is indented
            if let Some(shrd_node_indent_token) = node.indentation_token.as_ref() {
                let node_indent_slice = shrd_node_indent_token.slice;
                let node_indent_slice_len = node_indent_slice.len();

                let is_all_space = node_indent_slice.chars().all(|ch| ch == ' ');
                let is_all_tab = node_indent_slice.chars().all(|ch| ch == '\t');

                if !is_all_space && !is_all_tab {
                    self.add_diagnostic(
                        Diagnostic::new_error("Indentation must be entirely made up of either spaces or tabs, but not both")
                            .with_code("A:E0001")
                            .with_label(Label::new_primary(shrd_node_indent_token.span.clone()))
                    );

                    let node_id = arena.new_node(node);
                    if let Err(e) = parentless_sentinel_node_id.append(node_id, &mut arena) {
                        log::error!(
                            "Got an error attempting to make `{}` a child of `{}`: {:?}",
                            node_id,
                            parentless_sentinel_node_id,
                            e
                        );
                    }

                    // Since the indentation is bogus there is no reason
                    // to attempt to determine the parent (it'd just be a waste of cycles
                    // for a very small chance at being correct), just continue on to
                    // the next node.
                    continue;
                } else if is_all_space && node_indent_slice_len % SPACES_PER_INDENT_LEVEL != 0 {
                    self.add_diagnostic(
                        Diagnostic::new_error(format!(
                            "Column number must be a multiple of {} when using spaces",
                            SPACES_PER_INDENT_LEVEL
                        )).with_code("A:E0002")
                          .with_label(Label::new_primary(shrd_node_indent_token.span.clone()))
                    );

                    self.add_diagnostic(
                        Diagnostic::new_help(format!(
                            "Column number is currently {}",
                            node_indent_slice_len
                        ))
                    );
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

                                self.add_diagnostic(
                                    Diagnostic::new_bug(msg.clone())
                                        .with_code("A:E0003")
                                        .with_label(Label::new_primary(node.span().unwrap()))
                                );

                                panic!("{}", msg)
                            },
                        };

                        match calc_indent_delta(&shrd_last_parent_node.data, &node) {
                            IndentDelta::NoChange => {
                                let _sibling_id = parent_node_ids.pop(); // remove the sibling's ID
                                let node_span_opt = node.span();
                                let node_id = arena.new_node(node);

                                match parent_node_ids.last() {
                                    Some(shrd_last_parent_node_id) => {
                                        if let Err(e) = shrd_last_parent_node_id.append(node_id, &mut arena) {
                                            let err_msg = format!(
                                                "Got an error attempting to make `{}` a child of `{}`: {:?}",
                                                node_id,
                                                shrd_last_parent_node_id,
                                                e
                                            );

                                            log::error!("{}", err_msg);

                                            let mut diag = Diagnostic::new_bug(err_msg).with_code("A:E0006");

                                            if let Some(span) = node_span_opt {
                                                diag = diag.with_label(Label::new_primary(span));
                                            }

                                            self.add_diagnostic(diag);
                                        }
                                    },
                                    None => {
                                        let err_msg = format!(
                                            "Determined there was no indentation level change from the previous node, but no ID found in `{}`",
                                            stringify!(parent_node_ids),
                                        );

                                        log::error!("{}", err_msg);

                                        let mut diag = Diagnostic::new_bug(err_msg).with_code("A:E0007");

                                        if let Some(span) = node_span_opt {
                                            diag = diag.with_label(Label::new_primary(span));
                                        }

                                        self.add_diagnostic(diag);
                                    },
                                }

                                parent_node_ids.push(node_id);
                                all_node_ids.push(node_id);
                            },
                            IndentDelta::MoreIndented(col_num_diff) => {
                                if is_all_space && col_num_diff != SPACES_PER_INDENT_LEVEL {
                                    let mut diag = Diagnostic::new_error(format!(
                                        "Indentation difference must be {} spaces",
                                        SPACES_PER_INDENT_LEVEL
                                    )).with_code("A:E0004");

                                    if let Some(node_span) = node.indentation_token.as_ref().map(|token| token.span) {
                                        diag = diag.with_label(
                                            Label::new_primary(node_span)
                                        );
                                    }

                                    self.add_diagnostic(diag);

                                    self.add_diagnostic(
                                        Diagnostic::new_help(format!(
                                            "Consider deleting {} space(s)",
                                            col_num_diff
                                        ))
                                    );
                                } else if is_all_tab && col_num_diff != TABS_PER_INDENT_LEVEL {
                                    self.add_diagnostic(
                                        Diagnostic::new_error(format!(
                                            "Indentation difference must be {} tab(s)",
                                            TABS_PER_INDENT_LEVEL
                                        )).with_code("A:E0005")
                                    );
                                }

                                let node_id = arena.new_node(node);
                                parent_node_ids.push(node_id);
                                all_node_ids.push(node_id);

                                if let Err(e) = last_parent_node_id.append(node_id, &mut arena) {
                                    log::error!(
                                        "Got an error attempting to make `{}` a child of `{}`: {:?}",
                                        node_id,
                                        last_parent_node_id,
                                        e
                                    );
                                }
                            },
                            IndentDelta::LessIndented(_col_num_diff) => {
                                // Find the node that is 1 level unindented in reference to `node`
                                let parent_node_id_opt = {
                                    let desired_col_num_diff = if is_all_space {
                                        SPACES_PER_INDENT_LEVEL
                                    } else {
                                        TABS_PER_INDENT_LEVEL
                                    };

                                    let desired_indent_delta = IndentDelta::LessIndented(desired_col_num_diff);

                                    let mut iter = parent_node_ids.iter().rev();
                                    iter.find(|&&id| {
                                        let shrd_iter_node = match arena.get(id) {
                                            Some(n) => &n.data,
                                            _ => return false,
                                        };

                                        let indent_delta = calc_indent_delta(&node, shrd_iter_node);
                                        indent_delta == desired_indent_delta
                                    }).map(|id| *id)
                                };

                                if let Some(parent_node_id) = parent_node_id_opt {
                                    let parent_node_id_pos = parent_node_ids.iter().position(|&id| id == parent_node_id).unwrap();

                                    // The position we found earlier is 0-based but length is 1-based
                                    // so we must add 1 to the position when truncating.
                                    parent_node_ids.truncate(parent_node_id_pos + 1);

                                    let node_span_opt = node.span();
                                    let node_id = arena.new_node(node);
                                    parent_node_ids.push(node_id);
                                    all_node_ids.push(node_id);

                                    if let Err(e) = parent_node_id.append(node_id, &mut arena) {
                                        let err_msg = format!(
                                            "Got an error attempting to make `{}` a child of `{}`: {:?}",
                                            node_id,
                                            parent_node_id,
                                            e
                                        );

                                        log::error!("{}", err_msg);

                                        let mut diag = Diagnostic::new_bug(err_msg).with_code("A:E0008");

                                        if let Some(span) = node_span_opt {
                                            diag = diag.with_label(Label::new_primary(span));
                                        }

                                        self.add_diagnostic(diag);
                                    }
                                } else {
                                    self.add_diagnostic(
                                        Diagnostic::new_error("Unable to determine parent node due to indentation")
                                            .with_code("A:E0009")
                                            .with_label(Label::new_primary(node.span().unwrap()))
                                    );
                                }
                            },
                        }
                    },
                    None => {
                        self.add_diagnostic(
                            Diagnostic::new_error("Unable to determine parent node due to indentation")
                                .with_code("A:E0010")
                                .with_label(Label::new_primary(node.span().unwrap()))
                        );

                        let node_id = arena.new_node(node);
                        parent_node_ids.push(node_id);
                        all_node_ids.push(node_id);

                        if let Err(e) = parentless_sentinel_node_id.append(node_id, &mut arena) {
                            log::error!(
                                "Got an error attempting to make `{}` a child of `{}`: {:?}",
                                node_id,
                                parentless_sentinel_node_id,
                                e
                            );
                        }
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
            let n = &parentless_arena_node.data;
            if let Some(span) = n.span() {
                self.add_diagnostic(
                    Diagnostic::new_error("Unable to determine parent for this node, please address any prior errors and try again")
                        .with_label(Label::new_primary(span))
                );
            }
        }
        */

        Tree::new(all_node_ids, arena)
    }
}

#[cfg(test)]
mod tests;
