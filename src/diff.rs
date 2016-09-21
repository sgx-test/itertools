//! "Diff"ing iterators for caching elements to sequential collections without requiring the new
//! elements' iterator to be `Clone`.
//!
//! - [**Diff**](./enum.Diff.html) (produced by the [**diff_with**](./fn.diff_with.html) function)
//! describes the difference between two non-`Clone` iterators `I` and `J` after breaking ASAP from
//! a lock-step comparison.

use free::{put_back_with_value};
use structs::PutBack;

/// A type returned by the [`diff_with`](./fn.diff_with.html) function.
///
/// `Diff` represents the way in which the elements yielded by the iterator `I` differ to some
/// iterator `J`.
pub enum Diff<I, J>
    where I: Iterator,
          J: Iterator
{
    /// The index of the first non-matching element along with both iterator's remaining elements
    /// starting with the first mis-match.
    FirstMismatch(usize, PutBack<I>, PutBack<J>),
    /// The total number of elements that were in `J` along with the remaining elements of `I`.
    Shorter(usize, PutBack<I>),
    /// The total number of elements that were in `I` along with the remaining elements of `J`.
    Longer(usize, PutBack<J>),
}

/// Compares every element yielded by both `i` and `j` with the given function in lock-step and
/// returns a `Diff` which describes how `j` differs from `i`.
///
/// If the number of elements yielded by `j` is less than the number of elements yielded by `i`,
/// the number of `j` elements yielded will be returned along with `i`'s remaining elements as
/// `Diff::Shorter`.
///
/// If the two elements of a step differ, the index of those elements along with the remaining
/// elements of both `i` and `j` are returned as `Diff::FirstMismatch`.
///
/// If `i` becomes exhausted before `j` becomes exhausted, the number of elements in `i` along with
/// the remaining `j` elements will be returned as `Diff::Longer`.
pub fn diff_with<I, J, F>(i: I, j: J, is_equal: F)
    -> Option<Diff<I::IntoIter, J::IntoIter>>
    where I: IntoIterator,
          J: IntoIterator,
          F: Fn(&I::Item, &J::Item) -> bool
{
    let mut i = i.into_iter();
    let mut j = j.into_iter();
    let mut idx = 0;
    while let Some(i_elem) = i.next() {
        match j.next() {
            None => return Some(Diff::Shorter(idx, put_back_with_value(i_elem, i))),
            Some(j_elem) => if !is_equal(&i_elem, &j_elem) {
                let remaining_i = put_back_with_value(i_elem, i);
                let remaining_j = put_back_with_value(j_elem, j);
                return Some(Diff::FirstMismatch(idx, remaining_i, remaining_j));
            },
        }
        idx += 1;
    }
    j.next().map(|j_elem| Diff::Longer(idx, put_back_with_value(j_elem, j)))
}
