use std::{
    alloc::{alloc, dealloc, Layout},
    intrinsics::copy_nonoverlapping,
    ops::Deref,
    ptr::{drop_in_place, null},
    sync::atomic::{fence, AtomicUsize, Ordering},
};

const INITIAL_COUNT: usize = 0;

#[derive(Debug)]
#[repr(C)]
pub struct Arc<T> {
    pointer: *const T,
}

#[repr(C)]
struct ArcInner<T> {
    count: AtomicUsize,
    payload: T,
}

impl<T> Arc<T> {
    pub fn new(payload: T) -> Self {
        if Layout::new::<T>().size() == 0 {
            Self { pointer: null() }
        } else {
            let pointer = unsafe { &mut *(alloc(Self::inner_layout()) as *mut ArcInner<T>) };

            *pointer = ArcInner::<T> {
                count: AtomicUsize::new(INITIAL_COUNT),
                payload,
            };

            Self {
                pointer: &pointer.payload,
            }
        }
    }

    pub fn as_ptr(&self) -> *const T {
        &self.inner().payload
    }

    pub fn as_ptr_mut(&mut self) -> *mut T {
        &mut self.inner_mut().payload as *mut T
    }

    fn block_pointer(&self) -> *const ArcInner<T> {
        (unsafe { (self.pointer as *const usize).offset(-1) } as usize & !1) as *const ArcInner<T>
    }

    fn is_static(&self) -> bool {
        self.pointer as usize & 1 == 1
    }

    fn inner(&self) -> &ArcInner<T> {
        unsafe { &*self.block_pointer() }
    }

    fn inner_mut(&mut self) -> &mut ArcInner<T> {
        unsafe { &mut *(self.block_pointer() as *mut ArcInner<T>) }
    }

    fn inner_layout() -> Layout {
        Layout::new::<ArcInner<T>>()
    }
}

impl Arc<u8> {
    pub fn buffer(length: usize) -> Self {
        if length == 0 {
            Self { pointer: null() }
        } else {
            let pointer =
                unsafe { &mut *(alloc(Self::buffer_layout(length)) as *mut ArcInner<u8>) };

            pointer.count = AtomicUsize::new(INITIAL_COUNT);

            Self {
                pointer: &pointer.payload,
            }
        }
    }

    pub fn empty() -> Self {
        Self::buffer(0)
    }

    fn buffer_layout(length: usize) -> Layout {
        Layout::new::<ArcInner<()>>()
            .extend(Layout::array::<u8>(length).unwrap())
            .unwrap()
            .0
            .pad_to_align()
    }
}

impl<T> From<T> for Arc<T> {
    fn from(payload: T) -> Self {
        Self::new(payload)
    }
}

impl From<&[u8]> for Arc<u8> {
    fn from(slice: &[u8]) -> Self {
        let mut arc = Self::buffer(slice.len());

        unsafe { copy_nonoverlapping(slice.as_ptr(), arc.as_ptr_mut(), slice.len()) }

        arc
    }
}

impl From<Vec<u8>> for Arc<u8> {
    fn from(vec: Vec<u8>) -> Self {
        vec.as_slice().into()
    }
}

impl From<&str> for Arc<u8> {
    fn from(vec: &str) -> Self {
        vec.as_bytes().into()
    }
}

impl From<String> for Arc<u8> {
    fn from(string: String) -> Self {
        Vec::<u8>::from(string).into()
    }
}

impl<T> Deref for Arc<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.inner().payload
    }
}

impl<T> Clone for Arc<T> {
    fn clone(&self) -> Self {
        if !self.pointer.is_null() && !self.is_static() {
            // TODO Is this correct ordering?
            self.inner().count.fetch_add(1, Ordering::Relaxed);
        }

        Self {
            pointer: self.pointer,
        }
    }
}

impl<T> Drop for Arc<T> {
    fn drop(&mut self) {
        if self.pointer.is_null() || self.is_static() {
            return;
        }

        // TODO Is this correct ordering?
        if self.inner().count.fetch_sub(1, Ordering::Release) == INITIAL_COUNT {
            fence(Ordering::Acquire);

            unsafe {
                drop_in_place(self.as_ptr_mut());

                // TODO The layout is invalid for Arc<u8> buffer.
                dealloc(
                    self.inner() as *const ArcInner<T> as *mut u8,
                    Self::inner_layout(),
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn drop<T>(_: T) {}

    #[test]
    fn create() {
        Arc::new(0);
    }

    #[test]
    fn clone() {
        let arc = Arc::new(0);
        drop(arc.clone());
        drop(arc);
    }

    #[test]
    fn load_payload() {
        assert_eq!(*Arc::new(42), 42);
    }

    mod zero_sized {
        use super::*;

        #[test]
        fn create() {
            Arc::new(());
        }

        #[test]
        fn clone() {
            let arc = Arc::new(());
            drop(arc.clone());
            drop(arc);
        }

        #[test]
        #[allow(clippy::unit_cmp)]
        fn load_payload() {
            assert_eq!(*Arc::new(()), ());
        }
    }

    mod buffer {
        use super::*;

        #[test]
        fn create_buffer() {
            Arc::buffer(42);
        }

        #[test]
        fn create_zero_sized_buffer() {
            Arc::buffer(0);
        }

        #[test]
        fn clone() {
            let arc = Arc::buffer(42);
            drop(arc.clone());
            drop(arc);
        }

        #[test]
        fn convert_from_vec() {
            Arc::<u8>::from(vec![0u8; 42]);
        }

        #[test]
        fn convert_from_string() {
            Arc::<u8>::from("hello".to_string());
        }
    }
}
