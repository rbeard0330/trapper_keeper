use std::collections::HashMap;
use std::fmt::Debug;
use std::ops::{Deref, DerefMut, Index};
use crate::{Id, Key};

pub trait HeapItem: Debug + Clone {
    fn key(&self) -> Key;
    fn id(&self) -> Id;
}

impl<T: Clone + Into<Key> + Into<Id> + Debug> HeapItem for T {
    fn key(&self) -> Key {
        self.clone().into()
    }
    fn id(&self) -> Id {
        self.clone().into()
    }
}

#[derive(Debug)]
pub struct Heap<T: HeapItem> {
    heap: Vec<T>,
    index_map: HashMap<Id, usize>,
}

impl<T: HeapItem> Index<Id> for Heap<T> {
    type Output = T;

    fn index(&self, index: Id) -> &Self::Output {
        &self.heap[*self.index_map.get(&index).unwrap()]
    }
}

impl<T: HeapItem> Heap<T> {
    pub fn heapify(items: Vec<T>) -> Self {
        let item_count = items.len();
        let index_map = items.iter().enumerate().map(|(i, val)| (val.id(), i)).collect();
        let mut result = Heap {
            heap: items,
            index_map,
        };
        let mut view = result.get_mut_view();
        for ix in (0..(item_count >> 1)).rev() {
            view.index = ix;
            view.sift_down()
        }
        result
    }

    pub fn len(&self) -> usize {
        self.heap.len()
    }

    pub fn is_empty(&self) -> bool {
        self.heap.is_empty()
    }

    pub fn push(&mut self, value: T) {
        self.index_map.insert(value.id(), self.heap.len());
        self.heap.push(value);
        self.get_mut_view_at(self.heap.len() - 1).sift_up();
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.heap.is_empty() {
            None
        } else {
            let last_index = self.heap.len() - 1;
            self.get_mut_view().transpose(last_index);
            let result = self.heap.pop();
            self.index_map.remove(&result.as_ref().unwrap().id());
            self.get_mut_view().sift_down();
            result
        }
    }

    fn get_view(&self) -> HeapView<T> {
        self.get_view_at(0)
    }

    fn get_view_at(&self, index: usize) -> HeapView<T> {
        HeapView {
            index,
            heap: &self.heap,
            index_map: &self.index_map,
        }
    }

    fn get_mut_view(&mut self) -> HeapViewMut<T> {
        self.get_mut_view_at(0)
    }

    fn get_mut_view_at(&mut self, index: usize) -> HeapViewMut<T> {
        HeapViewMut {
            index,
            heap: &mut self.heap,
            index_map: &mut self.index_map,
        }
    }

    pub fn get_mut(&mut self, id: Id) -> Option<HeapItemRefMut<T>> {
        let index = *self.index_map.get(&id)?;
        let original_key = self.heap[index].key();
        let original_id = self.heap[index].id();
        Some(HeapItemRefMut {
            view: self.get_mut_view_at(index),
            original_key,
            original_id,
        })
    }

    pub fn get(&self, id: Id) -> Option<&T> {
        Some(&self.heap[*self.index_map.get(&id)?])
    }
}


#[derive(PartialEq, Debug)]
struct HeapView<'a, T: HeapItem> {
    index: usize,
    heap: &'a Vec<T>,
    index_map: &'a HashMap<Id, usize>,
}

impl<'a, T: HeapItem> HeapView<'a, T> {
    fn parent(&self) -> Option<Self> {
        if self.index == 0 {
            None
        } else {
            Some(HeapView { index: self.index >> 1, heap: self.heap, index_map: self.index_map })
        }
    }

    fn left(&self) -> Option<Self> {
        self.checked_from_index(2 * self.index + 1)
    }

    fn right(&self) -> Option<Self> {
        self.checked_from_index(2 * self.index + 2)
    }

    fn checked_from_index(&self, index: usize) -> Option<Self> {
        if index >= self.heap.len() {
            None
        } else {
            Some(
                HeapView {
                    index,
                    heap: self.heap,
                    index_map: self.index_map,
                })
        }
    }

    fn value(&self) -> &T {
        &self.heap[self.index]
    }
}


struct HeapViewMut<'a, T: HeapItem> {
    index: usize,
    heap: &'a mut Vec<T>,
    index_map: &'a mut HashMap<Id, usize>,
}

impl<'a, T: HeapItem> HeapViewMut<'a, T> {
    fn parent(&self) -> Option<usize> {
        if self.index == 0 {
            None
        } else {
            Some(self.index >> 1)
        }
    }

    fn left_index(&self) -> Option<usize> {
        let index = 2 * self.index + 1;
        if index < self.heap.len() {
            Some(index)
        } else {
            None
        }
    }

    fn right_index(&self) -> Option<usize> {
        let index = 2 * self.index + 2;
        if index < self.heap.len() {
            Some(index)
        } else {
            None
        }
    }

    fn sift_up(&mut self) {
        while let Some(parent_index) = self.parent() {
            if self.heap[parent_index].key() < self.heap[self.index].key() {
                self.transpose(parent_index)
            } else {
                break;
            }
        }
    }

    fn transpose(&mut self, dest: usize) {
        *self.index_map.get_mut(&self.heap[self.index].id()).unwrap() = dest;
        *self.index_map.get_mut(&self.heap[dest].id()).unwrap() = self.index;
        self.heap.swap(self.index, dest);
        self.index = dest;
    }

    fn sift_down(&mut self) {
        let left_index = self.left_index();
        let right_index = self.right_index();
        match (left_index, right_index) {
            (None, None) => {}
            (Some(left), Some(right)) => {
                let max = if self.heap[left].key() > self.heap[right].key() {
                    left
                } else {
                    right
                };
                if self.heap[self.index].key() < self.heap[max].key() {
                    self.transpose(max);
                    self.sift_down();
                }
            }
            (Some(index), None) | (None, Some(index)) => {
                if self.heap[self.index].key() < self.heap[index].key() {
                    self.transpose(index);
                    self.sift_down();
                }
            }
        }
    }
}

pub struct HeapItemRefMut<'a, T: HeapItem> {
    view: HeapViewMut<'a, T>,
    original_key: Key,
    original_id: Id,
}

impl<'a, T: HeapItem> Drop for HeapItemRefMut<'a, T> {
    fn drop(&mut self) {
        println!("restoring invariants when reference dropped");
        let new_id = self.view.heap[self.view.index].id();
        let new_key = self.view.heap[self.view.index].key();
        let (_, old_index) = self.view.index_map.remove_entry(&self.original_id).unwrap();
        debug_assert_eq!(old_index, self.view.index);
        self.view.index_map.insert(new_id, old_index);
        if self.original_key > new_key {
            self.view.sift_down();
        } else if self.original_key < new_key {
            self.view.sift_up();
        }
    }
}

impl<'a, T: HeapItem> Deref for HeapItemRefMut<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.view.heap.get(self.view.index).unwrap()
    }
}

impl<'a, T: HeapItem> DerefMut for HeapItemRefMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.view.heap.get_mut(self.view.index).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Formatter;
    use super::*;

    fn check_invariants<T: HeapItem>(heap: &Heap<T>) {
        for i in 0..heap.len() {
            let view = heap.get_view_at(i);
            if let Some(left) = view.left() {
                assert!(left.value().key() <= view.value().key());
            }
            if let Some(right) = view.right() {
                assert!(right.value().key() <= view.value().key());
            }
        }
        let view = heap.get_view();
        for (id, index) in view.index_map.iter() {
            assert_eq!(view.heap[*index].id(), *id);
        }
    }

    #[test]
    fn test_heapify() {
        let heap = Heap::heapify(vec![9, 8, 7, 6, 5, 4, 3, 2, 1]);
        check_invariants(&heap);
    }

    #[test]
    fn test_value() {
        let heap = Heap::heapify(vec![9, 8, 7, 6, 5, 4, 3, 2, 1]);
        check_invariants(&heap);
        let view = heap.get_view();
        assert_eq!(view.value(), &9);
    }

    #[test]
    fn left_works() {
        let heap = Heap::heapify(vec![9, 8, 7, 6, 5, 4, 3, 2, 1]);
        let view = heap.get_view();
        assert_eq!(view.left().unwrap().value(), &8);
        assert_eq!(view.left().unwrap().left().unwrap().value(), &6);
        assert_eq!(view.left().unwrap().left().unwrap().left().unwrap().value(), &2);
        assert_eq!(view.left().unwrap().left().unwrap().left().unwrap().left(), None);
    }

    #[test]
    fn right_works() {
        let heap = Heap::heapify(vec![9, 8, 7, 6, 5, 4, 3, 2, 1]);
        let view = heap.get_view();
        assert_eq!(view.right().unwrap().value(), &7);
        assert_eq!(view.right().unwrap().right().unwrap().value(), &3);
        assert_eq!(view.right().unwrap().right().unwrap().right(), None);
    }

    #[test]
    fn left_and_right_navigation() {
        let heap = Heap::heapify(vec![9, 8, 7, 6, 5, 4, 3, 2, 1]);
        let view = heap.get_view();
        assert_eq!(view.left().unwrap().left().unwrap().right().unwrap().value(), &1);
    }

    #[test]
    fn sift_up() {
        let mut heap = Heap::heapify(vec![1, 2, 3, 4, 5, 6, 7, 8, 9]);
        let mut view = heap.get_mut_view();
        view.sift_up();
        check_invariants(&heap);
    }

    #[test]
    fn pushes() {
        let mut heap = Heap::heapify(vec![]);
        heap.push(10);
        check_invariants(&heap);
        heap.push(1);
        check_invariants(&heap);
        heap.push(-100);
        check_invariants(&heap);
        heap.push(100);
        check_invariants(&heap);
        heap.push(12);
        check_invariants(&heap);
        heap.push(45);
    }

    #[test]
    fn pops() {
        let mut heap = Heap::heapify(vec![1, 2, 3, 4, 5, 6, 7, 8, 9]);
        check_invariants(&heap);
        while let Some(_) = heap.pop() {
            check_invariants(&heap);
        }
    }

    #[test]
    fn heap_sort() {
        let nums = vec![0, 100, 9, 41, -10, 55];
        let mut expected = nums.clone();
        expected.sort();
        expected.reverse();
        let mut heap = Heap::heapify(vec![]);
        for num in nums {
            heap.push(num);
            check_invariants(&heap);
        }
        let mut result = vec![];
        while let Some(num) = heap.pop() {
            result.push(num);
            check_invariants(&heap);
        }
        assert_eq!(expected, result);
    }

    #[test]
    fn test_invariants_restored() {
        let mut heap = Heap::heapify(vec![0, 100, 9, 41, -10, 55]);
        println!("before first modification");
        *heap.get_mut(-10).unwrap() = 200;
        println!("before second modification");
        *heap.get_mut(200).unwrap() = 300;
        println!("after all modifications, before read");
        assert_eq!(heap.heap[0], 300);
        println!("after read");
    }

    #[derive(Clone, Debug)]
    struct Job{
        priority: i64,
        id: i64,
        description: String
    }

    impl HeapItem for Job {
        fn key(&self) -> Key {
            self.priority
        }

        fn id(&self) -> Id {
            self.id
        }
    }

    #[test]
    fn test_invariants_restored_automatically() {
        let mut job_queue = Heap::heapify(vec![
            Job {id: 1, priority: 100, description: "Very urgent!".to_string()},
            Job {id: 2, priority: 50, description: "Medium urgent!".to_string()},
            Job {id: 3, priority: 0, description: "Meh, whenever".to_string()}
        ]);
        job_queue.pop().unwrap();
        println!("before modification");
        *job_queue.get_mut(3).unwrap() = Job {
            id: 3,
            priority: 200,
            description: "The boss wants this yesterday!".to_string()};
        println!("after modifications, before read");
        assert_eq!(&job_queue.pop().unwrap().description, "The boss wants this yesterday!");
        println!("after read");
    }
}
