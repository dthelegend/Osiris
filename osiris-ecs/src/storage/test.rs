#![cfg(test)]

use crate::storage::Table;
use std::cell::Cell;
use std::rc::Rc;

// Droopy things count how many times they have been dropped
struct Droopy(Rc<Cell<usize>>);

impl Drop for Droopy {
    fn drop(&mut self) {
        self.0.update(|x| x + 1)
    }
}

#[test]
fn extend_empty_table() {
    let mut sut = Table::new_for_bundle::<(Droopy,)>();
    let data = std::array::from_fn::<_, 1000, _>(|_| Rc::new(Cell::new(0)));

    assert_eq!(sut.len(), 0);
    assert_eq!(sut.capacity(), 0);

    sut.extend(data.iter().map(|x| (Droopy(x.clone()),)));

    assert!(sut.capacity() >= 1000);
    assert_eq!(sut.len(), 1000);

    sut.clear();

    for (idx, data) in data.iter().enumerate() {
        assert_eq!(data.get(), 1, "Value at {} is false!", idx);
    }
    // Congrats, all the items were dropped! And only dropped once!
}

#[test]
fn table_no_explicit_clear() {
    let data = std::array::from_fn::<_, 1000, _>(|_| Rc::new(Cell::new(0)));
    {
        let mut sut = Table::new_for_bundle::<(Droopy,)>();

        assert_eq!(sut.len(), 0);
        assert_eq!(sut.capacity(), 0);

        sut.extend(data.iter().map(|x| (Droopy(x.clone()),)));

        assert!(sut.capacity() >= 1000);
        assert_eq!(sut.len(), 1000);
    }

    for (idx, data) in data.iter().enumerate() {
        assert_eq!(data.get(), 1, "Value at {} is false!", idx);
    }
    // Congrats, all the items were dropped! And only dropped once!
}


#[test]
fn push_empty_table() {
    let mut sut = Table::new_for_bundle::<(Droopy,)>();
    let data = std::array::from_fn::<_, 1000, _>(|_| Rc::new(Cell::new(0)));

    assert_eq!(sut.len(), 0);
    assert_eq!(sut.capacity(), 0);

    for item in data.iter().map(|x| (Droopy(x.clone()),)) {
        sut.push(item);
    }

    assert!(sut.capacity() >= 1000);
    assert_eq!(sut.len(), 1000);

    sut.clear();

    for (idx, data) in data.iter().enumerate() {
        assert_eq!(data.get(), 1, "Value at {} is false!", idx);
    }
    // Congrats, all the items were dropped! And only dropped once!
}

#[test]
fn test_insert_at() {
    todo!()
}

#[test]
fn test_swap_remove() {
    todo!()
}

#[test]
fn test_pop() {
    todo!()
}

#[test]
fn test_remove_if() {
    todo!()
}
