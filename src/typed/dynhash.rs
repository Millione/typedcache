use std::any::Any;
use std::hash::{Hash, Hasher};

pub trait DynEq: Any {
    fn dyn_eq(&self, other: &dyn DynEq) -> bool;
    fn as_any(&self) -> &dyn Any;
    fn as_any_box(self: Box<Self>) -> Box<dyn Any>;
}

pub trait DynHash: DynEq {
    fn dyn_hash(&self, hasher: &mut dyn Hasher);
    fn as_dyn_eq(&self) -> &dyn DynEq;
}

impl<H: Eq + Any> DynEq for H {
    fn dyn_eq(&self, other: &dyn DynEq) -> bool {
        match other.as_any().downcast_ref::<H>() {
            Some(other) => self == other,
            None => false,
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_box(self: Box<Self>) -> Box<dyn Any> {
        self
    }
}

impl<H: Hash + DynEq> DynHash for H {
    fn dyn_hash(&self, mut hasher: &mut dyn Hasher) {
        H::hash(self, &mut hasher);
    }

    fn as_dyn_eq(&self) -> &dyn DynEq {
        self
    }
}

pub trait DowncastDynHash: Sized {
    fn downcast<T: Any>(self) -> Result<Box<T>, Self>;
}

impl DowncastDynHash for Box<dyn DynHash> {
    fn downcast<T: Any>(self) -> Result<Box<T>, Self> {
        if (*self).as_any().is::<T>() {
            let downcasted = self.as_any_box().downcast().expect("Broken Any downcast");
            Ok(downcasted)
        } else {
            Err(self)
        }
    }
}

impl PartialEq for dyn DynHash {
    fn eq(&self, other: &dyn DynHash) -> bool {
        self.dyn_eq(other.as_dyn_eq())
    }
}

impl Eq for dyn DynHash {}

impl Hash for dyn DynHash {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        self.dyn_hash(hasher);
    }
}
