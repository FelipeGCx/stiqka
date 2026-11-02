use std::sync::mpsc;
use std::thread;
use crate::logic::main::create_pack;
use crate::models::PackInfo;

pub struct Controller {
    progress: f32,
    is_processing: bool,
    result: Option<Result<String, String>>,
    receiver: Option<mpsc::Receiver<Result<String, String>>>,
    progress_receiver: Option<mpsc::Receiver<f32>>,
}

impl Controller {
    pub fn new() -> Self {
        Self {
            progress: 0.0,
            is_processing: false,
            result: None,
            receiver: None,
            progress_receiver: None,
        }
    }

    pub fn start(&mut self, pack_info: PackInfo) {
        self.is_processing = true;
        self.progress = 0.0;
        self.result = None;

        let (result_tx, result_rx) = mpsc::channel();
        self.receiver = Some(result_rx);

        let (progress_tx, progress_rx) = mpsc::channel();
        self.progress_receiver = Some(progress_rx);

        let on_progress = Some(Box::new(move |p: f32| {
            let _ = progress_tx.send(p);
        }) as Box<dyn Fn(f32) + Send + Sync>);

        thread::spawn(move || {
            let result = create_pack(pack_info, on_progress);
            let _ = result_tx.send(result);
        });
    }

    pub fn try_recv_progress(&mut self) -> Option<f32> {
        if let Some(rx) = &self.progress_receiver {
            if let Ok(p) = rx.try_recv() {
                return Some(p);
            }
        }
        None
    }

    pub fn update_progress(&mut self, progress: f32) {
        self.progress = progress.clamp(0.0, 1.0);
    }

    pub fn is_processing(&self) -> bool {
        self.is_processing
    }

    pub fn progress(&self) -> f32 {
        self.progress
    }

    pub fn try_take_result(&mut self) -> Option<Result<String, String>> {
        if let Some(receiver) = &self.receiver {
            if let Ok(result) = receiver.try_recv() {
                self.is_processing = false;
                self.progress = 1.0;
                self.result = Some(result);
                return self.result.take();
            }
        }
        self.result.take()
    }
}