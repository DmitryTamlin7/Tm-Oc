use core::{future::Future, pin::Pin, task::{Context, Poll}};
use alloc::boxed::Box;
use crate::interrupts::TICKS;
use core::sync::atomic::Ordering;

pub mod simple_executor;
pub mod keyboard;

pub struct Task {
    future: Pin<Box<dyn Future<Output = ()>>>,
}

impl Task {
    pub fn new(future: impl Future<Output = ()> + 'static) -> Task {
        Task {
            future: Box::pin(future),
        }
    }
    fn poll(&mut self, context: &mut Context) -> Poll<()> {
        self.future.as_mut().poll(context)
    }
}

pub fn sleep(ms: u64) -> SleepFuture {
    let target_tick = TICKS.load(Ordering::Relaxed) + ms;
    SleepFuture { target_tick }
}

pub struct SleepFuture {
    target_tick: u64,
}

impl Future for SleepFuture {
    type Output = ();
    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        if TICKS.load(Ordering::Relaxed) >= self.target_tick {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}