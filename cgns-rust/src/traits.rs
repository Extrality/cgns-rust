use core::ffi;

use crate::utils::Result;

pub trait CGNSNode<'a>: Sized {
    type Parent;

    fn id(&self) -> i32;
    fn from_id(parent: &'a Self::Parent, id: i32) -> Result<Self>;
    fn parent(&self) -> &Self::Parent;
}

pub trait CGNSParent<'a, C: CGNSNode<'a, Parent = Self>>: CGNSNode<'a> {
    fn num_child(&self) -> Result<i32>;

    fn iter(&'a self) -> Result<CGNSNodeIterator<'a, C>> {
        Ok(CGNSNodeIterator {
            parent: self,
            current: 0,
            len: self.num_child()?,
        })
    }
}

pub trait Read<'a, T>: CGNSNode<'a> {
    fn read(&self) -> Result<Vec<T>>;
}

pub struct CGNSNodeIterator<'a, C: CGNSNode<'a>> {
    parent: &'a C::Parent,
    current: ffi::c_int,
    len: ffi::c_int,
}

impl<'a, C: CGNSNode<'a>> Iterator for CGNSNodeIterator<'a, C> {
    type Item = C;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current == self.len {
            return None;
        }
        self.current += 1;
        match C::from_id(self.parent, self.current) {
            Ok(res) => Some(res),
            Err(e) => {
                println!("StopIter error: {e}");
                None
            }
        }
    }
}

impl<'a, C: CGNSNode<'a>> ExactSizeIterator for CGNSNodeIterator<'a, C> {
    fn len(&self) -> usize {
        self.len as usize - self.current as usize
    }
}
