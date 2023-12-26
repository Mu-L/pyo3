#[cfg(not(Py_LIMITED_API))]
use crate::ffi_ptr_ext::FfiPtrExt;
#[cfg(Py_LIMITED_API)]
use crate::types::PyIterator;
use crate::{
    err::{self, PyErr, PyResult},
    ffi, Bound, Py, PyAny, PyNativeType, PyObject, Python, ToPyObject,
};
use std::ptr;

/// Allows building a Python `frozenset` one item at a time
pub struct PyFrozenSetBuilder<'py> {
    py_frozen_set: &'py PyFrozenSet,
}

impl<'py> PyFrozenSetBuilder<'py> {
    /// Create a new `FrozenSetBuilder`.
    /// Since this allocates a `PyFrozenSet` internally it may
    /// panic when running out of memory.
    pub fn new(py: Python<'py>) -> PyResult<PyFrozenSetBuilder<'py>> {
        Ok(PyFrozenSetBuilder {
            py_frozen_set: PyFrozenSet::empty(py)?,
        })
    }

    /// Adds an element to the set.
    pub fn add<K>(&mut self, key: K) -> PyResult<()>
    where
        K: ToPyObject,
    {
        fn inner(frozenset: &PyFrozenSet, key: PyObject) -> PyResult<()> {
            err::error_on_minusone(frozenset.py(), unsafe {
                ffi::PySet_Add(frozenset.as_ptr(), key.as_ptr())
            })
        }

        inner(self.py_frozen_set, key.to_object(self.py_frozen_set.py()))
    }

    /// Finish building the set and take ownership of its current value
    pub fn finalize(self) -> &'py PyFrozenSet {
        self.py_frozen_set
    }
}

/// Represents a  Python `frozenset`
#[repr(transparent)]
pub struct PyFrozenSet(PyAny);

#[cfg(not(PyPy))]
pyobject_native_type!(
    PyFrozenSet,
    ffi::PySetObject,
    pyobject_native_static_type_object!(ffi::PyFrozenSet_Type),
    #checkfunction=ffi::PyFrozenSet_Check
);

#[cfg(PyPy)]
pyobject_native_type_core!(
    PyFrozenSet,
    pyobject_native_static_type_object!(ffi::PyFrozenSet_Type),
    #checkfunction=ffi::PyFrozenSet_Check
);

impl PyFrozenSet {
    /// Creates a new frozenset.
    ///
    /// May panic when running out of memory.
    #[inline]
    pub fn new<'a, 'p, T: ToPyObject + 'a>(
        py: Python<'p>,
        elements: impl IntoIterator<Item = &'a T>,
    ) -> PyResult<&'p PyFrozenSet> {
        new_from_iter(py, elements).map(|set| set.into_ref(py))
    }

    /// Creates a new empty frozen set
    pub fn empty(py: Python<'_>) -> PyResult<&PyFrozenSet> {
        unsafe { py.from_owned_ptr_or_err(ffi::PyFrozenSet_New(ptr::null_mut())) }
    }

    /// Return the number of items in the set.
    /// This is equivalent to len(p) on a set.
    #[inline]
    pub fn len(&self) -> usize {
        self.as_borrowed().len()
    }

    /// Check if set is empty.
    pub fn is_empty(&self) -> bool {
        self.as_borrowed().is_empty()
    }

    /// Determine if the set contains the specified key.
    /// This is equivalent to the Python expression `key in self`.
    pub fn contains<K>(&self, key: K) -> PyResult<bool>
    where
        K: ToPyObject,
    {
        self.as_borrowed().contains(key)
    }

    /// Returns an iterator of values in this frozen set.
    pub fn iter(&self) -> PyFrozenSetIterator<'_> {
        PyFrozenSetIterator(BoundFrozenSetIterator::new(self.as_borrowed().to_owned()))
    }
}

/// Implementation of functionality for [`PyFrozenSet`].
///
/// These methods are defined for the `Bound<'py, PyFrozenSet>` smart pointer, so to use method call
/// syntax these methods are separated into a trait, because stable Rust does not yet support
/// `arbitrary_self_types`.
#[doc(alias = "PyFrozenSet")]
pub trait PyFrozenSetMethods<'py> {
    /// Returns the number of items in the set.
    ///
    /// This is equivalent to the Python expression `len(self)`.
    fn len(&self) -> usize;

    /// Checks if set is empty.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Determines if the set contains the specified key.
    ///
    /// This is equivalent to the Python expression `key in self`.
    fn contains<K>(&self, key: K) -> PyResult<bool>
    where
        K: ToPyObject;

    /// Returns an iterator of values in this set.
    fn iter(&self) -> BoundFrozenSetIterator<'py>;
}

impl<'py> PyFrozenSetMethods<'py> for Bound<'py, PyFrozenSet> {
    #[inline]
    fn len(&self) -> usize {
        unsafe { ffi::PySet_Size(self.as_ptr()) as usize }
    }

    fn contains<K>(&self, key: K) -> PyResult<bool>
    where
        K: ToPyObject,
    {
        fn inner(frozenset: &Bound<'_, PyFrozenSet>, key: Bound<'_, PyAny>) -> PyResult<bool> {
            match unsafe { ffi::PySet_Contains(frozenset.as_ptr(), key.as_ptr()) } {
                1 => Ok(true),
                0 => Ok(false),
                _ => Err(PyErr::fetch(frozenset.py())),
            }
        }

        let py = self.py();
        inner(self, key.to_object(py).into_bound(py))
    }

    fn iter(&self) -> BoundFrozenSetIterator<'py> {
        BoundFrozenSetIterator::new(self.clone())
    }
}

/// PyO3 implementation of an iterator for a Python `frozenset` object.
pub struct PyFrozenSetIterator<'py>(BoundFrozenSetIterator<'py>);

impl<'py> Iterator for PyFrozenSetIterator<'py> {
    type Item = &'py super::PyAny;

    /// Advances the iterator and returns the next value.
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(Bound::into_gil_ref)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<'py> IntoIterator for &'py PyFrozenSet {
    type Item = &'py PyAny;
    type IntoIter = PyFrozenSetIterator<'py>;
    /// Returns an iterator of values in this set.
    fn into_iter(self) -> Self::IntoIter {
        PyFrozenSetIterator(BoundFrozenSetIterator::new(self.as_borrowed().to_owned()))
    }
}

impl<'py> IntoIterator for Bound<'py, PyFrozenSet> {
    type Item = Bound<'py, PyAny>;
    type IntoIter = BoundFrozenSetIterator<'py>;

    /// Returns an iterator of values in this set.
    fn into_iter(self) -> Self::IntoIter {
        BoundFrozenSetIterator::new(self)
    }
}

#[cfg(Py_LIMITED_API)]
mod impl_ {
    use super::*;

    /// PyO3 implementation of an iterator for a Python `set` object.
    pub struct BoundFrozenSetIterator<'p> {
        it: Bound<'p, PyIterator>,
    }

    impl<'py> BoundFrozenSetIterator<'py> {
        pub(super) fn new(frozenset: Bound<'py, PyFrozenSet>) -> Self {
            Self {
                it: PyIterator::from_object2(&frozenset).unwrap(),
            }
        }
    }

    impl<'py> Iterator for BoundFrozenSetIterator<'py> {
        type Item = Bound<'py, super::PyAny>;

        /// Advances the iterator and returns the next value.
        #[inline]
        fn next(&mut self) -> Option<Self::Item> {
            self.it.next().map(Result::unwrap)
        }
    }
}

#[cfg(not(Py_LIMITED_API))]
mod impl_ {
    use super::*;

    /// PyO3 implementation of an iterator for a Python `frozenset` object.
    pub struct BoundFrozenSetIterator<'py> {
        set: Bound<'py, PyFrozenSet>,
        pos: ffi::Py_ssize_t,
    }

    impl<'py> BoundFrozenSetIterator<'py> {
        pub(super) fn new(set: Bound<'py, PyFrozenSet>) -> Self {
            Self { set, pos: 0 }
        }
    }

    impl<'py> Iterator for BoundFrozenSetIterator<'py> {
        type Item = Bound<'py, PyAny>;

        #[inline]
        fn next(&mut self) -> Option<Self::Item> {
            unsafe {
                let mut key: *mut ffi::PyObject = std::ptr::null_mut();
                let mut hash: ffi::Py_hash_t = 0;
                if ffi::_PySet_NextEntry(self.set.as_ptr(), &mut self.pos, &mut key, &mut hash) != 0
                {
                    // _PySet_NextEntry returns borrowed object; for safety must make owned (see #890)
                    Some(key.assume_borrowed(self.set.py()).to_owned())
                } else {
                    None
                }
            }
        }

        #[inline]
        fn size_hint(&self) -> (usize, Option<usize>) {
            let len = self.len();
            (len, Some(len))
        }
    }

    impl<'py> ExactSizeIterator for BoundFrozenSetIterator<'py> {
        fn len(&self) -> usize {
            self.set.len().saturating_sub(self.pos as usize)
        }
    }

    impl<'py> ExactSizeIterator for PyFrozenSetIterator<'py> {
        fn len(&self) -> usize {
            self.0.len()
        }
    }
}

pub use impl_::*;

#[inline]
pub(crate) fn new_from_iter<T: ToPyObject>(
    py: Python<'_>,
    elements: impl IntoIterator<Item = T>,
) -> PyResult<Py<PyFrozenSet>> {
    fn inner(
        py: Python<'_>,
        elements: &mut dyn Iterator<Item = PyObject>,
    ) -> PyResult<Py<PyFrozenSet>> {
        let set: Py<PyFrozenSet> = unsafe {
            // We create the  `Py` pointer because its Drop cleans up the set if user code panics.
            Py::from_owned_ptr_or_err(py, ffi::PyFrozenSet_New(std::ptr::null_mut()))?
        };
        let ptr = set.as_ptr();

        for obj in elements {
            err::error_on_minusone(py, unsafe { ffi::PySet_Add(ptr, obj.as_ptr()) })?;
        }

        Ok(set)
    }

    let mut iter = elements.into_iter().map(|e| e.to_object(py));
    inner(py, &mut iter)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frozenset_new_and_len() {
        Python::with_gil(|py| {
            let set = PyFrozenSet::new(py, &[1]).unwrap();
            assert_eq!(1, set.len());

            let v = vec![1];
            assert!(PyFrozenSet::new(py, &[v]).is_err());
        });
    }

    #[test]
    fn test_frozenset_empty() {
        Python::with_gil(|py| {
            let set = PyFrozenSet::empty(py).unwrap();
            assert_eq!(0, set.len());
            assert!(set.is_empty());
        });
    }

    #[test]
    fn test_frozenset_contains() {
        Python::with_gil(|py| {
            let set = PyFrozenSet::new(py, &[1]).unwrap();
            assert!(set.contains(1).unwrap());
        });
    }

    #[test]
    fn test_frozenset_iter() {
        Python::with_gil(|py| {
            let set = PyFrozenSet::new(py, &[1]).unwrap();

            // iter method
            for el in set {
                assert_eq!(1i32, el.extract::<i32>().unwrap());
            }

            // intoiterator iteration
            for el in set {
                assert_eq!(1i32, el.extract::<i32>().unwrap());
            }
        });
    }

    #[test]
    #[cfg(not(Py_LIMITED_API))]
    fn test_frozenset_iter_size_hint() {
        Python::with_gil(|py| {
            let set = PyFrozenSet::new(py, &[1]).unwrap();
            let mut iter = set.iter();

            // Exact size
            assert_eq!(iter.len(), 1);
            assert_eq!(iter.size_hint(), (1, Some(1)));
            iter.next();
            assert_eq!(iter.len(), 0);
            assert_eq!(iter.size_hint(), (0, Some(0)));
        });
    }

    #[test]
    #[cfg(Py_LIMITED_API)]
    fn test_frozenset_iter_size_hint() {
        Python::with_gil(|py| {
            let set = PyFrozenSet::new(py, &[1]).unwrap();
            let iter = set.iter();

            // No known bounds
            assert_eq!(iter.size_hint(), (0, None));
        });
    }

    #[test]
    fn test_frozenset_builder() {
        use super::PyFrozenSetBuilder;

        Python::with_gil(|py| {
            let mut builder = PyFrozenSetBuilder::new(py).unwrap();

            // add an item
            builder.add(1).unwrap();
            builder.add(2).unwrap();
            builder.add(2).unwrap();

            // finalize it
            let set = builder.finalize();

            assert!(set.contains(1).unwrap());
            assert!(set.contains(2).unwrap());
            assert!(!set.contains(3).unwrap());
        });
    }
}
