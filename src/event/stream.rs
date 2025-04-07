use std::{
    io,
    pin::Pin,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, SyncSender},
        Arc,
    },
    task::{Context, Poll},
    thread,
    time::Duration,
};

use futures_core::Stream;

use super::{reader::EventReader, source::PlatformWaker, Event};

/// A stream of `termina::Event`s received from the terminal.
///
/// Create an event stream for a terminal with `termina::Terminal::event_stream`.
#[derive(Debug)]
pub struct EventStream<F: Fn(&Event) -> bool> {
    waker: PlatformWaker,
    filter: F,
    reader: EventReader,
    stream_wake_task_executed: Arc<AtomicBool>,
    stream_wake_task_should_shutdown: Arc<AtomicBool>,
    task_sender: SyncSender<Task>,
}

#[derive(Debug)]
struct Task {
    stream_waker: std::task::Waker,
    stream_wake_task_executed: Arc<AtomicBool>,
    stream_wake_task_should_shutdown: Arc<AtomicBool>,
}

impl<F> EventStream<F>
where
    F: Fn(&Event) -> bool + Clone + Send + Sync + 'static,
{
    pub(crate) fn new(reader: EventReader, filter: F) -> Self {
        let waker = reader.waker();

        let (task_sender, receiver) = mpsc::sync_channel::<Task>(1);

        let task_reader = reader.clone();
        let task_filter = filter.clone();
        thread::spawn(move || {
            while let Ok(task) = receiver.recv() {
                loop {
                    if let Ok(true) = task_reader.poll(None, &task_filter) {
                        break;
                    }
                    if task.stream_wake_task_should_shutdown.load(Ordering::SeqCst) {
                        break;
                    }
                }
                task.stream_wake_task_executed
                    .store(false, Ordering::SeqCst);
                task.stream_waker.wake();
            }
        });

        Self {
            waker,
            filter,
            reader,
            stream_wake_task_executed: Default::default(),
            stream_wake_task_should_shutdown: Default::default(),
            task_sender,
        }
    }
}

impl<F: Fn(&Event) -> bool> Drop for EventStream<F> {
    fn drop(&mut self) {
        self.stream_wake_task_should_shutdown
            .store(true, Ordering::SeqCst);
        let _ = self.waker.wake();
    }
}

impl<F: Fn(&Event) -> bool> Stream for EventStream<F> {
    type Item = io::Result<Event>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.reader.poll(Some(Duration::from_secs(0)), &self.filter) {
            Ok(true) => match self.reader.read(&self.filter) {
                Ok(event) => Poll::Ready(Some(Ok(event))),
                Err(err) => Poll::Ready(Some(Err(err))),
            },
            Ok(false) => {
                if !self
                    .stream_wake_task_executed
                    .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
                    .unwrap_or_else(|x| x)
                {
                    self.stream_wake_task_should_shutdown
                        .store(false, Ordering::SeqCst);
                    let _ = self.task_sender.send(Task {
                        stream_waker: cx.waker().clone(),
                        stream_wake_task_executed: self.stream_wake_task_executed.clone(),
                        stream_wake_task_should_shutdown: self
                            .stream_wake_task_should_shutdown
                            .clone(),
                    });
                }
                Poll::Pending
            }
            Err(err) => Poll::Ready(Some(Err(err))),
        }
    }
}

/// A dummy event stream which always polls as "pending."
///
/// This stream will never produce any events. This struct is meant for testing scenarios in
/// which you don't want to receive terminal events.
#[derive(Debug)]
pub struct DummyEventStream;

impl Stream for DummyEventStream {
    type Item = io::Result<Event>;

    fn poll_next(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Poll::Pending
    }
}
