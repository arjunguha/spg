use futures::channel::mpsc;
use futures::channel::oneshot;
use futures::prelude::*;
use inotify::Inotify;
use tokio::task;

/// Monitor the provided path for changes.
pub fn monitor_changes(path: &str) -> mpsc::Receiver<oneshot::Receiver<()>> {
    let (mut send, recv) = mpsc::channel(1);
    let mut inotify = Inotify::init().expect("initializing INotify");
    inotify
        .add_watch(path, inotify::WatchMask::MODIFY)
        .expect("watching path");
    // NOTE(arjun): It is surprising that I need to create this buffer here. Why doesn't the
    // API create it itself, and let me pick its size?
    let buffer = [0; 1024];
    let mut modify_events = inotify
        .event_stream(buffer)
        .expect("receiving INotify events");

    task::spawn(async move {
        loop {
            let (o_send, o_recv) = oneshot::channel();
            if let Err(_) = send.send(o_recv).await {
                break;
            }
            modify_events.next().await;
            o_send.send(()).unwrap();
        }
    });
    return recv;
}
