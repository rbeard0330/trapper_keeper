// use pyo3::{Py, PyAny};

mod heap;

pub use heap::{Heap, HeapItem};

type Key = i64;
type Id = i64;

struct PyWrapper {
    py_id: Id,
    key: Key,
    // object: Py<PyAny>
}
