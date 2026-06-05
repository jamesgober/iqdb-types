//! Attaching scalar metadata and building filter expressions over it.
//!
//! Run with: `cargo run --example metadata_and_filters`

use iqdb_types::{Filter, Metadata, Value};

fn main() {
    // --- Metadata: an immutable, ordered scalar map -----------------------
    let meta: Metadata = [
        ("author".to_string(), Value::String("ada".to_string())),
        ("year".to_string(), Value::Int(1843)),
        ("published".to_string(), Value::Bool(true)),
    ]
    .into_iter()
    .collect();

    println!("metadata has {} entries", meta.len());
    // BTreeMap-backed: iteration is always in ascending key order.
    for (key, value) in meta.iter() {
        println!("  {key} = {value:?}");
    }
    println!("year = {:?}", meta.get("year"));

    // --- Filters: a boolean expression tree over metadata -----------------
    // published == true AND year >= 1800 AND author IN {ada, grace}
    let filter = Filter::and(vec![
        Filter::eq("published", Value::Bool(true)),
        Filter::gte("year", Value::Int(1800)),
        Filter::is_in(
            "author",
            vec![
                Value::String("ada".to_string()),
                Value::String("grace".to_string()),
            ],
        ),
    ]);
    println!("filter = {filter:?}");

    // --- Closed-world semantics -------------------------------------------
    // A leaf comparison on an ABSENT field evaluates to `false`. So `neq` and
    // `not(eq)` are NOT interchangeable on records that lack the field:
    //   neq("author", "ada")     -> false for a record with no `author`
    //   not(eq("author", "ada")) -> true  for a record with no `author`
    let strict = Filter::neq("author", Value::String("ada".to_string()));
    let inclusive = Filter::not(Filter::eq("author", Value::String("ada".to_string())));
    println!("strict    (only records WITH a non-ada author): {strict:?}");
    println!("inclusive (also records WITHOUT an author):      {inclusive:?}");
}
