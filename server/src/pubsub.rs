use std::sync::{mpsc, Arc, Mutex};

pub struct PubSub<T: Clone> {
    senders: Arc<Mutex<Vec<mpsc::Sender<T>>>>,
}

impl<T: Clone> PubSub<T> {
    pub fn new() -> Self {
        PubSub {
            senders: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn subscribe(&self) -> mpsc::Receiver<T> {
        let (tx, rx) = mpsc::channel();
        self.senders.lock().unwrap().push(tx);
        rx
    }

    pub fn publish(&self, data: T) {
        let senders = self.senders.lock().unwrap();
        for sender in senders.iter() {
            sender.send(data.clone()).unwrap();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pubsub() {
        let pubsub = PubSub::new();
        let rx1 = pubsub.subscribe();
        let rx2 = pubsub.subscribe();

        pubsub.publish(42);

        assert_eq!(rx1.recv().unwrap(), 42);
        assert_eq!(rx2.recv().unwrap(), 42);
    }
}
