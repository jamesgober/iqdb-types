//! Building and inspecting vectors — the unit of value the iqdb spine indexes.
//!
//! Run with: `cargo run --example vectors`

use iqdb_types::{IqdbError, Vector, VectorRef};

fn main() -> Result<(), IqdbError> {
    // --- Owned, validated vectors -----------------------------------------
    let v = Vector::new(vec![0.1, 0.2, 0.3])?;
    println!("dim = {}, components = {:?}", v.dim(), v.as_slice());

    // Validation happens at construction. Empty and non-finite inputs are
    // rejected up front, so the rest of the spine never has to defend against
    // a bad vector.
    assert_eq!(
        Vector::new(Vec::new()).unwrap_err(),
        IqdbError::InvalidVector
    );
    assert_eq!(
        Vector::new(vec![1.0, f32::NAN]).unwrap_err(),
        IqdbError::InvalidVector,
    );

    // `try_into` is the ergonomic alias for `Vector::new`.
    let w: Vector = vec![1.0, 0.0, 0.0].try_into()?;
    println!("w dim = {}", w.dim());

    // Reclaim the underlying buffer when you need the owned `Vec` back.
    let buf = w.into_inner();
    println!("reclaimed {} components", buf.len());

    // --- Borrowed views (zero-copy) ---------------------------------------
    // `VectorRef` borrows a slice — pass a query vector through an API without
    // giving up ownership or allocating a copy.
    let query = [0.5_f32, 0.5, 0.0];
    let r = VectorRef::from(&query[..]);
    println!("query dim = {}, first = {}", r.dim(), r.as_slice()[0]);

    Ok(())
}
