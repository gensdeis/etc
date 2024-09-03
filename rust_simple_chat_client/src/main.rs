use eframe::{egui, App, NativeOptions};
use tokio::net::TcpStream;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::{mpsc, Mutex};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), eframe::Error> {
    let options = NativeOptions::default();
    eframe::run_native(
        "Chat Client",
        options,
        Box::new(|_cc| Ok(Box::new(ChatApp::default()) as Box<dyn App>)),
    )
}

struct ChatApp {
    messages: Vec<String>,
    input_text: String,
    server_ip: String,
    server_port: String,
    connected: bool,
    sender: Option<mpsc::Sender<String>>,
    receiver: Option<Arc<Mutex<mpsc::Receiver<String>>>>,
}

impl Default for ChatApp {
    fn default() -> Self {
        Self {
            messages: Vec::new(),
            input_text: String::new(),
            server_ip: "127.0.0.1".to_string(),
            server_port: "8080".to_string(),
            connected: false,
            sender: None,
            receiver: None,
        }
    }
}

impl App for ChatApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Chat Client");

            if !self.connected {
                ui.horizontal(|ui| {
                    ui.label("Server IP:");
                    ui.text_edit_singleline(&mut self.server_ip);
                });

                ui.horizontal(|ui| {
                    ui.label("Port:");
                    ui.text_edit_singleline(&mut self.server_port);
                });

                if ui.button("Connect").clicked() {
                    let ip = self.server_ip.clone();
                    let port = self.server_port.clone();
                    let (sender, receiver) = mpsc::channel(100);
                    let receiver = Arc::new(Mutex::new(receiver));

                    // Spawn a new task to handle the TCP connection
                    let sender_clone = sender.clone();
                    let receiver_clone = Arc::clone(&receiver);

                    tokio::spawn(async move {
                        let addr = format!("{}:{}", ip, port);
                        match TcpStream::connect(&addr).await {
                            Ok(mut stream) => {
                                let (reader, _writer) = stream.split();
                                let mut reader = BufReader::new(reader);
                                let mut line = String::new();

                                while let Ok(bytes) = reader.read_line(&mut line).await {
                                    if bytes == 0 {
                                        break;
                                    }
                                    if let Err(e) = sender_clone.send(line.clone()).await {
                                        eprintln!("Failed to send message: {}", e);
                                    }
                                    line.clear();
                                }
                            }
                            Err(e) => eprintln!("Connection failed: {}", e),
                        }
                    });

                    self.sender = Some(sender);
                    self.receiver = Some(receiver);
                    self.connected = true;
                }
            } else {
                ui.horizontal(|ui| {
                    ui.text_edit_singleline(&mut self.input_text);
                    if ui.button("Send").clicked() {
                        if let Some(sender) = &self.sender {
                            let msg = self.input_text.clone();
                            self.input_text.clear();
                            let sender_clone = sender.clone();
                            tokio::spawn(async move {
                                if let Err(e) = sender_clone.send(msg).await {
                                    eprintln!("Failed to send message: {}", e);
                                }
                            });
                        }
                    }
                });

                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.label("Chat:");
                    for msg in &self.messages {
                        ui.label(msg);
                    }
                });
            }
        });

        // Poll for new messages and update the UI
        if let Some(receiver) = &self.receiver {
            let receiver_clone = Arc::clone(receiver);
            tokio::spawn(async move {
                let mut receiver = receiver_clone.lock().await;
                while let Some(msg) = receiver.recv().await {
                    // Append new messages to the `messages` vector
                    // Use `tokio::sync::mpsc::Sender` to pass the message to the main thread
                    // This example just prints the message for simplicity
                    println!("Received message: {}", msg);
                    // Here you would need to update the `messages` vector in the main thread
                    // Consider using a channel to pass this data to the main thread
                }
            });
        }
    }
}
