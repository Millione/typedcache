use std::any::Any;

pub type TypedValue = TypedValueBase<dyn Any + 'static + Send + Sync>;

pub struct TypedValueBase<T: ?Sized + 'static + Any = dyn Any + 'static>(Box<T>);

impl TypedValueBase<dyn Any + Send + Sync + 'static> {
    pub fn from_value<V: Any + Send + Sync + 'static>(value: V) -> Self {
        Self(Box::new(value))
    }
    
    #[must_use]
    pub fn downcast<V: Any>(self) -> Option<V> {
        let boxed: Box<dyn Any + 'static> = self.0;
        let boxed: Option<Box<V>> = boxed.downcast().ok();
        boxed.map(|v| *v)
    }

    #[must_use]
    pub fn downcast_ref<V: Any>(&self) -> Option<&V> {
        self.0.as_ref().downcast_ref::<V>()
    }

    pub fn downcast_mut<V: Any>(&mut self) -> Option<&mut V> {
        self.0.as_mut().downcast_mut::<V>()
    }
}
