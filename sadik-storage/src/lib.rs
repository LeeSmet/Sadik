use std::collections::HashMap;
use std::fmt;
use std::future::Future;
use std::rc::Rc;
use std::sync::RwLock;
use std::task::Poll;

pub enum NamespaceType {
    User,
    System,
}

pub struct Placement<'a> {
    pub namespace_type: NamespaceType,
    pub namespace: &'a [u8],
    pub key: &'a [u8],
}

pub trait Storage<'a> {
    type SetOutput: Future<Output = Result<(), Self::Error>> + 'a;
    type GetOutput: Future<Output = Result<Option<Vec<u8>>, Self::Error>> + 'a;
    type Error: std::error::Error;

    fn get(&self, placement: Placement<'a>) -> Self::GetOutput;
    fn set(&self, placement: Placement<'a>, data: &'a [u8]) -> Self::SetOutput;
}

#[derive(Default)]
pub struct MapStorage {
    backend: Rc<RwLock<HashMap<Vec<u8>, Vec<u8>>>>,
}

impl MapStorage {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<'a> Storage<'a> for MapStorage {
    type Error = MapStorageError;
    type SetOutput = MapStorageSetOutput<'a>;
    type GetOutput = MapStorageGetOutput<'a>;

    fn get(&self, placement: Placement<'a>) -> Self::GetOutput {
        MapStorageGetOutput {
            placement,
            db: self.backend.clone(),
        }
    }
    fn set(&self, placement: Placement<'a>, data: &'a [u8]) -> Self::SetOutput {
        MapStorageSetOutput {
            placement,
            data,
            db: self.backend.clone(),
        }
    }
}

pub struct MapStorageSetOutput<'a> {
    placement: Placement<'a>,
    data: &'a [u8],
    db: Rc<RwLock<HashMap<Vec<u8>, Vec<u8>>>>,
}
impl<'a> Future for MapStorageSetOutput<'a> {
    type Output = Result<(), MapStorageError>;

    fn poll(self: std::pin::Pin<&mut Self>, _: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        self.db
            .write()
            .expect("Nothing here panicks")
            .insert(contruct_storage_key(&self.placement), Vec::from(self.data));

        Poll::Ready(Ok(()))
    }
}

pub struct MapStorageGetOutput<'a> {
    placement: Placement<'a>,
    db: Rc<RwLock<HashMap<Vec<u8>, Vec<u8>>>>,
}
impl<'a> Future for MapStorageGetOutput<'a> {
    type Output = Result<Option<Vec<u8>>, MapStorageError>;

    fn poll(self: std::pin::Pin<&mut Self>, _: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        Poll::Ready(Ok(self
            .db
            .read()
            .expect("Nothing here panicks")
            .get(&contruct_storage_key(&self.placement))
            .map(Vec::clone)))
    }
}

fn contruct_storage_key(placement: &Placement) -> Vec<u8> {
    let mut key = Vec::with_capacity(1 + 4 + placement.namespace.len() + placement.key.len());
    key.push(match placement.namespace_type {
        NamespaceType::User => 0,
        NamespaceType::System => 1,
    });
    key.extend_from_slice(&placement.namespace.len().to_be_bytes());
    key.extend_from_slice(placement.namespace);
    key.extend_from_slice(placement.key);
    key
}

#[derive(Debug)]
pub struct MapStorageError {}

impl fmt::Display for MapStorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad("some_error_thingy")
    }
}

impl std::error::Error for MapStorageError {}

#[cfg(test)]
mod tests {

    #[test]
    fn test_set_and_get() {
        use super::{MapStorage, NamespaceType, Placement, Storage};
        use futures::task::noop_waker_ref;
        use std::future::Future;
        use std::pin::Pin;
        use std::task::{Context, Poll};

        const NS: &[u8; 4] = b"test";
        const KEY: &[u8; 7] = b"somekey";
        const DATA: &[u8; 43] = b"this is some test data for testing purposes";

        let mut cx = Context::from_waker(noop_waker_ref());

        let storage = MapStorage::new();
        let set_placement = Placement {
            namespace_type: NamespaceType::User,
            namespace: NS,
            key: KEY,
        };

        let mut set_fut = storage.set(set_placement, DATA);
        let res = Pin::new(&mut set_fut).poll(&mut cx);
        assert!(matches!(res, Poll::Ready(_)));

        let get_placement = Placement {
            namespace_type: NamespaceType::User,
            namespace: NS,
            key: KEY,
        };

        let mut get_fut = storage.get(get_placement);
        let res = Pin::new(&mut get_fut).poll(&mut cx);
        match res {
            Poll::Pending => panic!("MapStorage operations can't be pending"),
            Poll::Ready(stored_data) => {
                assert_eq!(DATA.as_ref(), &stored_data.unwrap().unwrap());
            }
        }

        let get2_placement = Placement {
            namespace_type: NamespaceType::System,
            namespace: NS,
            key: KEY,
        };

        let mut get2_fut = storage.get(get2_placement);
        let res = Pin::new(&mut get2_fut).poll(&mut cx);
        match res {
            Poll::Pending => panic!("MapStorage operations can't be pending"),
            Poll::Ready(stored_data) => {
                assert!(stored_data.unwrap().is_none());
            }
        }
    }
}
