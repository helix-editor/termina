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

use super::{
    reader::InternalEventReader,
    source::{EventSource, PlatformEventSource, PlatformWaker},
    Event, InternalEvent,
};

#[derive(Debug)]
pub struct EventStream {
    waker: PlatformWaker,
    reader: InternalEventReader,
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

impl EventStream {
    pub(crate) fn new(source: PlatformEventSource) -> Self {
        let waker = source.waker();
        let reader = InternalEventReader::new(source);

        let (task_sender, receiver) = mpsc::sync_channel::<Task>(1);

        let task_reader = reader.clone();
        let filter =
            |internal_event: &InternalEvent| matches!(internal_event, InternalEvent::Event(_));
        thread::spawn(move || {
            while let Ok(task) = receiver.recv() {
                loop {
                    if let Ok(true) = task_reader.poll(None, filter) {
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
            reader,
            stream_wake_task_executed: Default::default(),
            stream_wake_task_should_shutdown: Default::default(),
            task_sender,
        }
    }
}

impl Drop for EventStream {
    fn drop(&mut self) {
        self.stream_wake_task_should_shutdown
            .store(true, Ordering::SeqCst);
        let _ = self.waker.wake();
    }
}

impl Stream for EventStream {
    type Item = io::Result<Event>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let filter =
            |internal_event: &InternalEvent| matches!(internal_event, InternalEvent::Event(_));

        match self.reader.poll(Some(Duration::from_secs(0)), filter) {
            Ok(true) => match self.reader.read(filter) {
                Ok(InternalEvent::Event(event)) => Poll::Ready(Some(Ok(event))),
                Err(err) => Poll::Ready(Some(Err(err))),
                // _ => unreachable!(),
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
