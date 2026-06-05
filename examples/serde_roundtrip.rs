//! Serializing and deserializing the public types (requires the `serde` feature).
//!
//! Run with: `cargo run --example serde_roundtrip --features serde`

use iqdb_types::{DistanceMetric, Filter, Hit, Metadata, SearchParams, Value, Vector, VectorId};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // A fully described search request, round-tripped through JSON.
    let params = SearchParams {
        ef: Some(64),
        filter: Some(Filter::and(vec![
            Filter::eq("published", Value::Bool(true)),
            Filter::gte("year", Value::Int(2020)),
        ])),
        ..SearchParams::new(5, DistanceMetric::Cosine)
    };
    let json = serde_json::to_string_pretty(&params)?;
    println!("SearchParams as JSON:\n{json}");
    let back: SearchParams = serde_json::from_str(&json)?;
    assert_eq!(params, back);

    // Every public data type round-trips losslessly.
    let vector = Vector::new(vec![0.1, 0.2, 0.3])?;
    let id = VectorId::try_from(vec![0xde, 0xad, 0xbe, 0xef])?;
    let meta: Metadata = [("k".to_string(), Value::Int(1))].into_iter().collect();
    let hit = Hit {
        id: id.clone(),
        distance: 0.5,
        metadata: Some(meta.clone()),
    };

    assert_eq!(
        vector,
        serde_json::from_str::<Vector>(&serde_json::to_string(&vector)?)?,
    );
    assert_eq!(
        id,
        serde_json::from_str::<VectorId>(&serde_json::to_string(&id)?)?,
    );
    assert_eq!(
        meta,
        serde_json::from_str::<Metadata>(&serde_json::to_string(&meta)?)?,
    );
    assert_eq!(
        hit,
        serde_json::from_str::<Hit>(&serde_json::to_string(&hit)?)?,
    );

    println!("\nall public types round-tripped through JSON ✔");
    Ok(())
}
