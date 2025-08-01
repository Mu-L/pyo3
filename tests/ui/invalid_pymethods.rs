use pyo3::prelude::*;

#[pyclass]
struct MyClass {}

#[pymethods]
impl MyClass {
    #[classattr]
    fn class_attr_with_args(_foo: i32) {}

    #[classattr(foobar)]
    const CLASS_ATTR_WITH_ATTRIBUTE_ARG: i32 = 3;

    fn staticmethod_without_attribute() {}

    #[staticmethod]
    fn staticmethod_with_receiver(&self) {}

    #[classmethod]
    fn classmethod_with_receiver(&self) {}

    #[classmethod]
    fn classmethod_missing_argument() -> Self {
        Self {}
    }
}

struct NotATypeObject;

#[pymethods]
impl MyClass {
    #[classmethod]
    fn classmethod_wrong_first_argument(_t: NotATypeObject) -> Self {
        Self {}
    }
}

#[pymethods]
impl MyClass {
    #[getter(x)]
    fn getter_without_receiver() {}
}

#[pymethods]
impl MyClass {
    #[setter(x)]
    fn setter_without_receiver() {}
}

#[pymethods]
impl MyClass {
    #[pyo3(name = "__call__", text_signature = "()")]
    fn text_signature_on_call() {}

    #[getter(x)]
    #[pyo3(text_signature = "()")]
    fn text_signature_on_getter(&self) {}

    #[setter(x)]
    #[pyo3(text_signature = "()")]
    fn text_signature_on_setter(&self) {}

    #[classattr]
    #[pyo3(text_signature = "()")]
    fn text_signature_on_classattr() {}

    #[pyo3(text_signature = 1)]
    fn invalid_text_signature() {}

    #[pyo3(text_signature = "()")]
    #[pyo3(text_signature = None)]
    fn duplicate_text_signature() {}
}

#[pymethods]
impl MyClass {
    #[getter(x)]
    #[pyo3(signature = ())]
    fn signature_on_getter(&self) {}

    #[setter(x)]
    #[pyo3(signature = ())]
    fn signature_on_setter(&self) {}

    #[classattr]
    #[pyo3(signature = ())]
    fn signature_on_classattr() {}
}

#[pymethods]
impl MyClass {
    #[new]
    #[classmethod]
    #[staticmethod]
    #[classattr]
    #[getter(x)]
    #[setter(x)]
    fn multiple_method_types() {}
}

#[pymethods]
impl MyClass {
    #[new(signature = ())]
    fn new_takes_no_arguments(&self) {}
}

#[pymethods]
impl MyClass {
    #[new = ()] // in this form there's no suggestion to move arguments to `#[pyo3()]` attribute
    fn new_takes_no_arguments_nv(&self) {}
}

#[pymethods]
impl MyClass {
    #[classmethod(signature = ())]
    fn classmethod_takes_no_arguments(&self) {}
}

#[pymethods]
impl MyClass {
    #[staticmethod(signature = ())]
    fn staticmethod_takes_no_arguments(&self) {}
}

#[pymethods]
impl MyClass {
    #[classattr(signature = ())]
    fn classattr_takes_no_arguments(&self) {}
}

#[pymethods]
impl MyClass {
    fn generic_method<T>(_value: T) {}
}

#[pymethods]
impl MyClass {
    fn impl_trait_method_first_arg(_impl_trait: impl AsRef<PyAny>) {}

    fn impl_trait_method_second_arg(&self, _impl_trait: impl AsRef<PyAny>) {}
}

#[pymethods]
impl MyClass {
    #[pyo3(pass_module)]
    fn method_cannot_pass_module(&self, _m: &PyModule) {}
}

#[pymethods]
impl MyClass {
    fn method_self_by_value(self) {}
}

macro_rules! macro_invocation {
    () => {};
}

#[pymethods]
impl MyClass {
    macro_invocation!();
}

#[pymethods]
impl MyClass {
    #[staticmethod]
    #[classmethod]
    fn multiple_errors_static_and_class_method() {}

    #[staticmethod]
    fn multiple_errors_staticmethod_with_receiver(&self) {}

    #[classmethod]
    fn multiple_errors_classmethod_with_receiver(&self) {}
}

fn main() {}
