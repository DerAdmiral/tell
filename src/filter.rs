use std::marker::PhantomData;


pub struct FilterPredicate<X> {
    __wraping_type: PhantomData<X>,
    predicate: Box<dyn Fn(&X) -> bool>,
}

impl<X, F: Fn(&X) -> bool + 'static> From<F> for FilterPredicate<X> {
    fn from(value: F) -> Self {
        Self::new(value)
    }
}

impl<X> FilterPredicate<X> {
    pub fn new<P: Fn(&X) -> bool + 'static>(p: P) -> Self {
        Self { __wraping_type: PhantomData, predicate: Box::new(p) }
    }

    pub fn apply(&self, x: &X) -> bool {
        (self.predicate)(x)
    }
}