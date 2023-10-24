use std::{marker::PhantomData, num::NonZeroUsize};

use bevy::{prelude::Resource, utils::HashMap};

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Id<T: ?Sized + Send + Sync>(NonZeroUsize, PhantomData<fn() -> *const T>);

impl<T: ?Sized + Send + Sync> Clone for Id<T> {
    fn clone(&self) -> Self {
        Self(self.0, self.1)
    }
}

impl<T: ?Sized + Send + Sync> Copy for Id<T> {}

#[derive(Resource, Clone)]
pub struct Registry<T: ?Sized + Send + Sync> {
    data: Vec<Box<T>>,
    names: HashMap<String, Id<T>>,
}

impl<T: ?Sized + Send + Sync> Registry<T> {
    pub fn register(&mut self, name: String, value: Box<T>) -> Id<T> {
        let id = Id(NonZeroUsize::new(self.data.len() + 1).unwrap(), PhantomData);
        self.data.push(value);
        self.names.insert(name, id);
        id
    }

    pub fn id(&self, name: &str) -> Option<Id<T>> {
        self.names.get(name).copied()
    }

    pub fn block(&self, id: Id<T>) -> &T {
        self.data[id.0.get() - 1].as_ref()
    }
}

impl<T: ?Sized + Send + Sync> Default for Registry<T> {
    fn default() -> Self {
        Self {
            data: Vec::new(),
            names: HashMap::new(),
        }
    }
}
