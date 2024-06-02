use alloc::{collections::VecDeque, vec::Vec};
use core::pin::Pin;
use core::task::{Context, Poll, Waker};
use core::{cell::RefCell, future::poll_fn};
use defmt::warn;
use embassy_sync::blocking_mutex::{raw::RawMutex, Mutex};
use futures_util::Stream;

pub struct PubSubChannel<M: RawMutex, T: Clone> {
    inner: Mutex<M, RefCell<PubSubState<T>>>,
}

impl<M: RawMutex, T: Clone> PubSubChannel<M, T> {
    pub const fn new() -> Self {
        Self {
            inner: Mutex::const_new(M::INIT, RefCell::new(PubSubState::new())),
        }
    }

    pub fn publish(&self, message: T) {
        self.inner.lock(|inner| inner.borrow_mut().publish(message));
    }

    pub fn publisher(&self) -> Publisher<'_, M, T> {
        Publisher { channel: self }
    }

    pub fn subscriber(&self) -> Subscriber<'_, M, T> {
        self.inner.lock(|inner| {
            let mut s = inner.borrow_mut();
            s.subscriber_count += 1;
            Subscriber {
                channel: self,
                next_message_id: s.next_message_id,
            }
        })
    }

    pub fn clear(&self) {
        self.inner.lock(|inner| inner.borrow_mut().queue.clear());
    }

    pub fn len(&self) -> usize {
        self.inner.lock(|inner| inner.borrow().queue.len())
    }

    pub fn is_empty(&self) -> bool {
        self.inner.lock(|inner| inner.borrow().queue.is_empty())
    }

    fn get_message_with_context(
        &self,
        next_message_id: &mut u64,
        cx: Option<&mut Context<'_>>,
    ) -> Poll<WaitResult<T>> {
        self.inner.lock(|s| {
            let mut s = s.borrow_mut();

            // Check if we can read a message
            match s.get_message(*next_message_id) {
                // Yes, so we are done polling
                Some(WaitResult::Message(message)) => {
                    *next_message_id += 1;
                    Poll::Ready(WaitResult::Message(message))
                }
                // No, so we need to reregister our waker and sleep again
                None => {
                    if let Some(cx) = cx {
                        let new_waker = cx.waker();
                        let mut waker_is_present = false;
                        for waker in &s.subscriber_wakers {
                            if new_waker.will_wake(waker) {
                                waker_is_present = true;
                            }
                        }
                        if !waker_is_present {
                            s.subscriber_wakers.push(new_waker.clone());
                        }
                    }
                    Poll::Pending
                }
                // We missed a couple of messages. We must do our internal bookkeeping and return that we lagged
                Some(WaitResult::Lagged(amount)) => {
                    *next_message_id += amount;
                    Poll::Ready(WaitResult::Lagged(amount))
                }
            }
        })
    }

    fn available(&self, next_message_id: u64) -> u64 {
        self.inner
            .lock(|s| s.borrow().next_message_id - next_message_id)
    }

    fn unregister_subscriber(&self, subscriber_next_message_id: u64) {
        self.inner.lock(|s| {
            let mut s = s.borrow_mut();
            s.unregister_subscriber(subscriber_next_message_id);
        });
    }
}

impl<M: RawMutex, T: Clone> Default for PubSubChannel<M, T> {
    fn default() -> Self {
        Self::new()
    }
}

struct PubSubState<T: Clone> {
    queue: VecDeque<(T, usize)>,
    next_message_id: u64,
    subscriber_wakers: Vec<Waker>,
    subscriber_count: usize,
}

impl<T: Clone> PubSubState<T> {
    const fn new() -> Self {
        Self {
            queue: VecDeque::new(),
            next_message_id: 0,
            subscriber_wakers: Vec::new(),
            subscriber_count: 0,
        }
    }

    fn publish(&mut self, message: T) {
        if self.subscriber_count == 0 {
            return;
        }
        self.queue.push_back((message, self.subscriber_count));
        self.next_message_id += 1;

        for waker in self.subscriber_wakers.drain(..) {
            waker.wake();
        }
    }

    fn get_message(&mut self, message_id: u64) -> Option<WaitResult<T>> {
        let start_id = self.next_message_id - self.queue.len() as u64;

        if message_id < start_id {
            return Some(WaitResult::Lagged(start_id - message_id));
        }

        let current_message_index = (message_id - start_id) as usize;

        if current_message_index >= self.queue.len() {
            return None;
        }

        let queue_item = self.queue.get_mut(current_message_index).unwrap();

        queue_item.1 -= 1;

        let message = if current_message_index == 0 && queue_item.1 == 0 {
            let (message, _) = self.queue.pop_front().unwrap();
            message
        } else {
            queue_item.0.clone()
        };

        Some(WaitResult::Message(message))
    }

    fn unregister_subscriber(&mut self, subscriber_next_message_id: u64) {
        self.subscriber_count -= 1;

        // All messages that haven't been read yet by this subscriber must have their counter decremented
        let start_id = self.next_message_id - self.queue.len() as u64;
        if subscriber_next_message_id >= start_id {
            let current_message_index = (subscriber_next_message_id - start_id) as usize;
            self.queue
                .iter_mut()
                .skip(current_message_index)
                .for_each(|(_, counter)| *counter -= 1);

            while let Some((_, count)) = self.queue.front() {
                if *count == 0 {
                    self.queue.pop_front().unwrap();
                } else {
                    break;
                }
            }
        }
    }
}

enum WaitResult<T> {
    Lagged(u64),
    Message(T),
}

pub struct Publisher<'a, M: RawMutex, T: Clone> {
    channel: &'a PubSubChannel<M, T>,
}

impl<'a, M: RawMutex, T: Clone> Publisher<'a, M, T> {
    pub fn publish(&self, message: T) {
        self.channel.publish(message);
    }
}

impl<'a, M: RawMutex, T: Clone> Copy for Publisher<'a, M, T> {}

impl<'a, M: RawMutex, T: Clone> Clone for Publisher<'a, M, T> {
    fn clone(&self) -> Self {
        *self
    }
}

pub struct Subscriber<'a, M: RawMutex, T: Clone> {
    channel: &'a PubSubChannel<M, T>,
    next_message_id: u64,
}

impl<'a, M: RawMutex, T: Clone> Subscriber<'a, M, T> {
    async fn next_message_inner(&mut self) -> WaitResult<T> {
        poll_fn(|cx| {
            self.channel
                .get_message_with_context(&mut self.next_message_id, Some(cx))
        })
        .await
    }

    pub async fn next_message(&mut self) -> T {
        loop {
            match self.next_message_inner().await {
                WaitResult::Lagged(v) => {
                    warn!("Subscriber lagged by {} messages", v);
                    continue;
                }
                WaitResult::Message(value) => return value,
            }
        }
    }

    pub fn available(&self) -> u64 {
        self.channel.available(self.next_message_id)
    }
}

impl<'a, M: RawMutex, T: Clone> Drop for Subscriber<'a, M, T> {
    fn drop(&mut self) {
        self.channel.unregister_subscriber(self.next_message_id);
    }
}

impl<'a, M: RawMutex, T: Clone> Stream for Subscriber<'a, M, T> {
    type Item = T;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self
            .channel
            .get_message_with_context(&mut self.next_message_id, Some(cx))
        {
            Poll::Ready(WaitResult::Message(message)) => Poll::Ready(Some(message)),
            Poll::Ready(WaitResult::Lagged(_)) => {
                cx.waker().wake_by_ref();
                Poll::Pending
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

impl<'a, M: RawMutex, T: Clone> Clone for Subscriber<'a, M, T> {
    fn clone(&self) -> Self {
        Self {
            channel: self.channel,
            next_message_id: self.next_message_id,
        }
    }
}
