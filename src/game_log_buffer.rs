use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use tauri::Emitter;
use tokio::io::{AsyncBufReadExt, AsyncRead, BufReader};
use tokio::sync::mpsc;
use tokio::time::{Duration, interval};

// Constants for performance tuning
const MAX_LOG_LINES: usize = 5000; // Prevents memory leaks/infinite growth
const TICK_INTERVAL_MS: u64 = 100; // Lutris "sweet spot" for smoothness

#[derive(Debug, Clone)]
pub struct LogBuffer {
    lines: VecDeque<String>, // Changed to VecDeque for efficient popping
    total_lines_ever: usize,
}

impl LogBuffer {
    pub fn new() -> Self {
        Self {
            lines: VecDeque::with_capacity(MAX_LOG_LINES),
            total_lines_ever: 0,
        }
    }

    pub fn append_lines(&mut self, new_lines: Vec<String>) {
        for line in new_lines {
            if self.lines.len() >= MAX_LOG_LINES {
                self.lines.pop_front();
            }
            self.lines.push_back(line);
            self.total_lines_ever += 1;
        }
    }


    pub fn get_all(&self) -> String {
        // This efficiently joins the VecDeque without unnecessary clones
        self.lines.iter().map(|s| s.as_str()).collect::<Vec<_>>().join("\n")
    }
}

pub struct LogBufferManager {
    buffers: Arc<Mutex<HashMap<String, Arc<Mutex<LogBuffer>>>>>,
}

impl LogBufferManager {
    pub fn new() -> Self {
        Self {
            buffers: Arc::new(Mutex::new(HashMap::new())),
        }
    }


    pub fn get(&self, slug: &str) -> Option<Arc<Mutex<LogBuffer>>> {
        let buffers = self.buffers.lock().unwrap();
        buffers.get(slug).cloned()
    }

    pub fn get_or_create(&self, slug: &str) -> Arc<Mutex<LogBuffer>> {
        let mut buffers = self.buffers.lock().unwrap();
        buffers
            .entry(slug.to_string())
            .or_insert_with(|| Arc::new(Mutex::new(LogBuffer::new())))
            .clone()
    }

    pub fn remove(&self, slug: &str) {
        let mut buffers = self.buffers.lock().unwrap();
        buffers.remove(slug);
    }
}

#[derive(Clone, serde::Serialize)]
pub struct LogChunkPayload {
    pub slug: String,
    pub lines: Vec<String>, // We send the raw list to the frontend
    pub total_lines: usize,
}

pub struct LogStreamer {
    slug: String,
    buffer: Arc<Mutex<LogBuffer>>,
}

impl LogStreamer {
    pub fn new(slug: String, buffer: Arc<Mutex<LogBuffer>>) -> Self {
        Self { slug, buffer }
    }

    pub async fn stream_output<R>(self, reader: R, window: tauri::Window)
    where
        R: AsyncRead + Unpin + Send + 'static,
    {
        let (tx, mut rx) = mpsc::channel::<String>(1000); // Buffer up to 1000 lines
        let slug = self.slug.clone();
        let buffer_clone = self.buffer.clone();

        tokio::spawn(async move {
            let mut lines = BufReader::new(reader).lines();
            println!("[DEBUG] Producer started for {}", self.slug.clone().to_string()); // CHECKPOINT 1
            while let Ok(Some(line)) = lines.next_line().await {
                // If the channel is full, we wait (backpressure)
                if tx.send(line).await.is_err() {
                    break;
                }
            }
            println!("[DEBUG] Producer finished for {}", self.slug.clone().to_string());
        });


        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_millis(TICK_INTERVAL_MS));

            loop {
                ticker.tick().await;

                let mut batched_lines = Vec::new();

                // 1. DRAIN: Take all lines currently waiting in the channel
                while let Ok(line) = rx.try_recv() {
                    batched_lines.push(line);
                }

                // 2. PROCESS: If we actually got new data, update the buffer and notify UI
                if !batched_lines.is_empty() {
                    // We lock the buffer only long enough to append and get the new total
                    let current_total = {
                        let mut buf = buffer_clone.lock().unwrap();
                        buf.append_lines(batched_lines.clone());
                        buf.total_lines_ever
                    };

                    // 3. EMIT: Send the batch to the frontend
                    let _ = window.emit(&format!("game-log:{}", slug), LogChunkPayload {
                        slug: slug.clone(),
                        lines: batched_lines, // The frontend receives just the new lines
                        total_lines: current_total, // Useful for the frontend to track sync
                    });
                }

                // 4. EXIT: Shutdown if the producer task finished and the channel is empty
                if rx.is_closed() && rx.try_recv().is_err() {
                    break;
                }
            }
        });
    }
}