# Parsing: Incremental Recomputation

What is _incremental recomputation_?

This is difficult for me to explain concisely.

I highly suggest watching [a video created](https://www.youtube.com/watch?v=_muY4HjSqVw)
by the primary author/maintainer of `salsa` (the third-party package that gives us this functionality).

The [`salsa` docs](https://github.com/salsa-rs/salsa#key-idea) are helpful, too.

---

Basically this lets us compute _something_ once and memoize the result which
means the next time we compute it we can just get the result from the cache
if all of the computation's dependencies remain unchanged.

Think about computing a tree for a given file path and look at the following graph-like illustration.

Arrow direction indicates "has a dependency on", with `foo() <- bar()` meaning "`bar` has a dependency on `foo`".

```
file_text("infantry.yaml") <- file_tree("infantry.yaml") <----+
                                                         | <---- type-checking
file_text("vehicles.yaml") <- file_tree("vehicles.yaml") <----+
```

If the result of `file_text("infantry.yaml")` changes do we need to reperform the `type-checking` computation?

The answer to that is "if the result of `file_tree("infantry.yaml")` changed."

If the text changed but the tree remains the same then all of `type-checking`'s dependencies remain unchanged so we do not need to actually type-check again, we can reuse the last `type-check`'s results.

---

## How does this package implement this functionality?

The `oraide-parser-miniyaml` package exports some types that make this simple:

- `OraideDatabase`, contains memoized computation results
- `ParserCtx`, defines inputs and queries for a `OraideDatabase`
- `ParserCtxExt`, helper functions that make using `OraideDatabase` a bit easier

### Example

```rust
use oraide_parser_miniyaml::{
    OraideDatabase,
    ParserCtx,
    ParserCtxExt,
    Tree,
};

// this is *your function*, not implemented in `oraide-parser-miniyaml`
let file_contents: String = get_contents_of("example.yaml");

let mut db = OraideDatabase::default();

// the first argument is a string used to identify a file by humans,
// in most cases it will probably be the path to the file
//
// `db.add_file` is a helper function introduced with `ParserCtxExt`
// that both queries and sets inputs on the database.
//
// you should read its implementation to understand more
let file_id = db.add_file("example.yaml", file_contents);

// `db.file_tree` is a *query* since you are *querying* the database for a tree given a file ID
let tree: Tree = db.file_tree(file_id);
```

Now if we were to call `db.file_tree(file_id)` again (with the same `file_id`
value) _without any relevant inputs having changed_ (such as the file's text)
we would quickly return the same results that were computed the first time it
was invoked.

> You may recall that in another document we said we'd get to `FileId` creation, well now you've seen it (`db.add_file` returns a new `FileId`)