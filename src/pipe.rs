use std::io;
use std::string::FromUtf8Error;
use std::sync::mpsc::{channel, Receiver};
use std::thread::spawn;

#[derive(Debug)]
pub enum PipeError {
    IO(io::Error),
    NotUtf8(FromUtf8Error),
}

#[derive(Debug)]
pub enum PipedLine {
    Line(String),
    EOF,
}

// Reads data from the pipe byte-by-byte and returns the lines.
// Useful for processing the pipe's output as soon as it becomes available.
pub struct PipeStreamReader {
    pub lines: Receiver<Result<PipedLine, PipeError>>,
}

impl PipeStreamReader {
    // Starts a background task reading bytes from the pipe.
    pub fn new(mut stream: Box<dyn io::Read + Send>) -> PipeStreamReader {
        PipeStreamReader {
            lines: {
                let (tx, rx) = channel();

                spawn(move || {
                    let mut buf = Vec::new();
                    let mut byte = [0u8];
                    loop {
                        match stream.read(&mut byte) {
                            Ok(0) => {
                                let _ = tx.send(Ok(PipedLine::EOF));
                                break;
                            }
                            Ok(_) => {
                                if byte[0] == 0x0A {
                                    tx.send(match String::from_utf8(buf.clone()) {
                                        Ok(line) => Ok(PipedLine::Line(line)),
                                        Err(err) => Err(PipeError::NotUtf8(err)),
                                    })
                                    .unwrap();
                                    buf.clear()
                                } else {
                                    buf.push(byte[0])
                                }
                            }
                            Err(error) => {
                                tx.send(Err(PipeError::IO(error))).unwrap();
                            }
                        }
                    }
                });

                rx
            },
        }
    }
}
