pub trait ToBox<T> {
    fn to_box(&self) -> Box<T>;
}

impl<T> ToBox<T> for T
where
    T: Clone,
{
    fn to_box(&self) -> Box<T> {
        Box::new(self.clone())
    }
}

impl<T: crate::ToValue> crate::ToValue for std::rc::Rc<T> {
    fn to_value(&self) -> crate::Value<'_> {
        self.as_ref().to_value()
    }
}
