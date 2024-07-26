use std::ptr;
use std::task::{RawWaker, RawWakerVTable, Waker};

#[test]
fn test_waker_getters() {
    let raw_waker = RawWaker::new(ptr::without_provenance_mut(42usize), &WAKER_VTABLE);
    assert_eq!(raw_waker.data() as usize, 42);
    assert!(ptr::eq(raw_waker.vtable(), &WAKER_VTABLE));

    let waker = unsafe { Waker::from_raw(raw_waker) };
    let waker2 = waker.clone();
    let raw_waker2 = waker2.as_raw();
    assert_eq!(raw_waker2.data() as usize, 43);
    assert!(ptr::eq(raw_waker2.vtable(), &WAKER_VTABLE));
}

static WAKER_VTABLE: RawWakerVTable = RawWakerVTable::new(
    |data| RawWaker::new(ptr::without_provenance_mut(data as usize + 1), &WAKER_VTABLE),
    |_| {},
    |_| {},
    |_| {},
);

// https://github.com/rust-lang/rust/issues/102012#issuecomment-1915282956
mod nop_waker {
    use core::{
        future::{ready, Future},
        pin::Pin,
        task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
    };

    const NOP_RAWWAKER: RawWaker = {
        fn nop(_: *const ()) {}
        const VTAB: RawWakerVTable = RawWakerVTable::new(|_| NOP_RAWWAKER, nop, nop, nop);
        RawWaker::new(&() as *const (), &VTAB)
    };

    const NOP_WAKER: &Waker = &unsafe { Waker::from_raw(NOP_RAWWAKER) };

    const NOP_CONTEXT: Context<'static> = Context::from_waker(NOP_WAKER);

    fn poll_once<T, F>(f: &mut F) -> Poll<T>
    where
        F: Future<Output = T> + ?Sized + Unpin,
    {
        let mut cx = NOP_CONTEXT;
        Pin::new(f).as_mut().poll(&mut cx)
    }

    #[test]
    fn test_const_waker() {
        assert_eq!(poll_once(&mut ready(1)), Poll::Ready(1));
    }
}
