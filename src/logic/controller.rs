use std::sync::mpsc;
use std::thread;
use crate::logic::main::create_pack;
use crate::models::PackInfo;

pub struct Controller {
    progress: f32,
    is_processing: bool,
    result: Option<Result<String, String>>,
    receiver: Option<mpsc::Receiver<Result<String, String>>>,
}

impl Controller {
    pub fn new() -> Self {
        Self {
            progress: 0.0,
            is_processing: false,
            result: None,
            receiver: None,
        }
    }

    pub fn start(&mut self, pack_info: PackInfo) {
        self.is_processing = true;
        self.progress = 0.0;
        self.result = None;

        let (sender, receiver) = mpsc::channel();
        self.receiver = Some(receiver);

        thread::spawn(move || {
            let result = create_pack(pack_info);
            let _ = sender.send(result);
        });
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