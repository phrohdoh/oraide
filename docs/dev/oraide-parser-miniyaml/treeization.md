# Parsing: The Treeization Phase

What is "treeization"?

The process of determining relationships between a collection of `Node`s.

In the current implementation a text document becomes a single `Tree`, not multiple.

---

To start we [create a collection of parent node IDs](../../../components/oraide-parser-miniyaml/src/parser/treeizer/mod.rs#L274-L426) which keeps a hierarchy of potential parents.

We also have [a collection of _all_ node IDs](../../../components/oraide-parser-miniyaml/src/parser/treeizer/mod.rs#L176) that is part of the resulting `Tree` structure.

The first node of this `Tree` is a so-called [sentinel](https://en.wikipedia.org/wiki/Sentinel_value).

If a node is encountered but a parent cannot be determined the node will be parented to this sentinel (see [this code](../../../components/oraide-parser-miniyaml/src/parser/treeizer/mod.rs#L428-L447)).

We create an ID for the current node by adding it to the underlying node storage.

Any time a node is added to the underlying node storage we push its ID onto the `all_node_ids` collection.

Any time a non-comment-only node is added to the underlying node storage we push its ID onto the `parent_node_ids` collection.

## Determining A Node's Parent

When we are attempting to determine a node's parent we look at the last item in the `parent_node_ids` collection and [compare its indentation level against the current node's indentation level](../../../components/oraide-parser-miniyaml/src/parser/treeizer/mod.rs#L428-L447).

There are 3 possible results:

- **no change in indentation**
    - the current node should be made a sibling of the comparee
        - do this by popping the sibling's ID off of `parent_node_ids`, then parent to the last item in that collection
- **the current node is _more indented_ than the comparee**
    - an indentation level difference of 1 means this node should be a child of the comparee
        - do this by pushing this node's ID onto the end of `parent_node_ids`
    - an indentation level difference of 2 or more is invalid, parent the current node to the sentinel
- **the current node is _less indented_ than the comparee**
    - if the indentation level is 0 this node is _top-level_ (starting a new actor/weapon/whatever definition)
        - clear `parent_node_ids`
    - otherwise [iterate over `parent_node_ids` _backwards_ to find a node that has 1 less indentation level compared to the current node](../../../components/oraide-parser-miniyaml/src/parser/treeizer/mod.rs#L367-L386)
        - if such a node is found, truncate `parent_node_ids` down to that node's position in the collection, then parent the current node to the just-found parent node
        - if such a node is not found parent the current node to the sentinel

At the end `all_node_ids` and the underlying node storage (`arena`) are used to create and return a `Tree` structure.