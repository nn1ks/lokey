use alloc::collections::VecDeque;
use core::cell::RefCell;
use core::future::poll_fn;
use core::task::{Context, Poll};
use embassy_sync::blocking_mutex::{raw::RawMutex, Mutex};
use embassy_sync::waitqueue::WakerRegistration;

struct State<T> {
    queue: VecDeque<T>,
    receiver_waker: WakerRegistration,
}

impl<T> State<T> {
    fn send(&mut self, message: T) {
        self.queue.push_back(message);
        self.receiver_waker.wake();
    }

    fn poll_receive(&mut self, cx: &Context) -> Poll<T> {
        match self.queue.pop_front() {
            Some(message) => Poll::Ready(message),
            None => {
                self.receiver_waker.register(cx.waker());
                Poll::Pending
            }
        }
    }
}

pub struct Channel<M, T> {
    inner: Mutex<M, RefCell<State<T>>>,
}

impl<M: RawMutex, T> Channel<M, T> {
    pub const fn new() -> Self {
        let state = State {
            queue: VecDeque::new(),
            receiver_waker: WakerRegistration::new(),
        };
        Self {
            inner: Mutex::new(RefCell::new(state)),
        }
    }

    pub fn send(&self, message: T) {
        self.inner.lock(|state| {
            state.borrow_mut().send(message);
        })
    }

    pub fn poll_receive(&self, cx: &Context) -> Poll<T> {
        self.inner.lock(|state| state.borrow_mut().poll_receive(cx))
    }

    pub async fn receive(&self) -> T {
        poll_fn(|cx| self.inner.lock(|state| state.borrow_mut().poll_receive(cx))).await
    }
}

impl<M: RawMutex, T> Default for Channel<M, T> {
    fn default() -> Self {
        Self::new()
    }
}
