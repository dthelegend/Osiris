#![cfg(test)]

use crate::storage::Table;
use std::cell::Cell;
use std::rc::Rc;

// Droopy things count how many times they have been dropped
struct Droopy(isize, Rc<Cell<usize>>);

impl Drop for Droopy {
    fn drop(&mut self) {
        self.0 = -1;
        self.1.update(|x| x + 1)
    }
}

#[test]
fn extend_empty_table() {
    let mut sut = Table::new_for_bundle::<(Droopy,)>();
    let data = std::array::from_fn::<_, 1000, _>(|_| Rc::new(Cell::new(0)));

    assert_eq!(sut.len(), 0);
    assert_eq!(sut.capacity(), 0);

    sut.extend(data.iter().enumerate().map(|(idx, x)| (Droopy(idx as isize, x.clone()),)));

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

        sut.extend(data.iter().enumerate().map(|(idx, x)| (Droopy(idx as isize, x.clone()),)));

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

    for item in data.iter().enumerate().map(|(idx, x)| (Droopy(idx as isize, x.clone()),)) {
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
    let mut sut = Table::new_for_bundle::<(Droopy,)>();
    let data = std::array::from_fn::<_, 1000, _>(|_| Rc::new(Cell::new(0)));

    assert_eq!(sut.len(), 0);
    assert_eq!(sut.capacity(), 0);

    for item in data.iter().enumerate().take(999).map(|(idx, x)| (Droopy(idx as isize, x.clone()),)) {
        sut.push(item);
    }

    assert!(sut.capacity() >= 999);
    assert_eq!(sut.len(), 999);

    const DROOPSTER_IDX : usize = 999 / 2;
    let old_droopster = sut.insert_at(DROOPSTER_IDX, (Droopy(999, data[999].clone()),));

    drop(old_droopster);

    assert_eq!(data[DROOPSTER_IDX].get(), 1);

    sut.clear();

    for (idx, data) in data.iter().enumerate() {
        assert_eq!(data.get(), 1, "Value at {} is false!", idx);
    }
    // Congrats, all the items were dropped! And only dropped once!
}

#[test]
fn test_swap_remove() {
    let mut sut = Table::new_for_bundle::<(Droopy,)>();
    let data = std::array::from_fn::<_, 1000, _>(|_| Rc::new(Cell::new(0)));

    assert_eq!(sut.len(), 0);
    assert_eq!(sut.capacity(), 0);

    for item in data.iter().enumerate().map(|(idx, x)| (Droopy(idx as isize, x.clone()),)) {
        sut.push(item);
    }

    assert!(sut.capacity() >= 1000);
    assert_eq!(sut.len(), 1000);

    const MAX_IDX : usize = 1000 / 2;

    for droopster in 0..MAX_IDX {
        sut.swap_remove(droopster);
    }

    for (idx, data) in data.iter().take(MAX_IDX).enumerate() {
        assert_eq!(data.get(), 1, "Value at {} is false!", idx);
    }

    sut.clear();

    for (idx, data) in data.iter().enumerate() {
        assert_eq!(data.get(), 1, "Value at {} is false!", idx);
    }
    // Congrats, all the items were dropped! And only dropped once!
}

#[test]
fn test_swap_pop() {
    let mut sut = Table::new_for_bundle::<(Droopy,)>();
    let data = std::array::from_fn::<_, 1000, _>(|_| Rc::new(Cell::new(0)));

    assert_eq!(sut.len(), 0);
    assert_eq!(sut.capacity(), 0);

    for item in data.iter().enumerate().map(|(idx, x)| (Droopy(idx as isize, x.clone()),)) {
        sut.push(item);
    }

    assert!(sut.capacity() >= 1000);
    assert_eq!(sut.len(), 1000);

    const MAX_IDX : usize = 1000 / 2;

    for droopster in 0..MAX_IDX {
        let (popped,) = sut.swap_pop::<(Droopy,)>(droopster);
        assert_eq!(popped.0, droopster as isize);
        assert_eq!(popped.1.get(), 0);
    }

    for (idx, data) in data.iter().take(MAX_IDX).enumerate() {
        assert_eq!(data.get(), 1, "Value at {} is false!", idx);
    }

    sut.clear();

    for (idx, data) in data.iter().enumerate() {
        assert_eq!(data.get(), 1, "Value at {} is false!", idx);
    }
    // Congrats, all the items were dropped! And only dropped once!
}

#[test]
fn test_pop() {
    let mut sut = Table::new_for_bundle::<(Droopy,)>();
    let data = std::array::from_fn::<_, 1000, _>(|_| Rc::new(Cell::new(0)));

    assert_eq!(sut.len(), 0);
    assert_eq!(sut.capacity(), 0);

    for item in data.iter().enumerate().map(|(idx, x)| (Droopy(idx as isize, x.clone()),)) {
        sut.push(item);
    }

    assert!(sut.capacity() >= 1000);
    assert_eq!(sut.len(), 1000);

    const MIN_IDX: usize = 1000 / 2;

    for droopster in (MIN_IDX..1000).rev() {
        let (popped,) = sut.pop::<(Droopy,)>();
        assert_eq!(popped.0, droopster as isize);
        assert_eq!(popped.1.get(), 0);
    }

    for (idx, data) in data.iter().enumerate().skip(MIN_IDX) {
        assert_eq!(data.get(), 1, "Value at {} is false!", idx);
    }

    sut.clear();

    for (idx, data) in data.iter().enumerate() {
        assert_eq!(data.get(), 1, "Value at {} is false!", idx);
    }
    // Congrats, all the items were dropped! And only dropped once!
}

#[test]
fn test_remove_if() {
}
