use std::future::Future;
use std::sync::{Arc, atomic::AtomicBool, atomic::Ordering};
use std::time::Duration;
use ctrlc::Signal;
use log::info;
use anyhow::{Result, Context};
use signal_hook::consts::{SIGWINCH, SIGINT, SIGTERM, SIGHUP};
use signal_hook::iterator::Signals;
use signal_hook::consts::signal;
use std::thread;

#[derive(Debug, Clone)]
pub struct OsNotifier {
    ctrlc: Arc<AtomicBool>,
    kill: Arc<AtomicBool>,
}

impl OsNotifier {
    pub fn new() -> Result<OsNotifier> {
        let mut s = OsNotifier {
            kill: Arc::new(AtomicBool::new(false)),
            ctrlc: Arc::new(AtomicBool::new(false)),
        };

        s.register()?;

        Ok(s)
    }

    pub fn is_kill_shutdown(&self) -> bool {
        self.kill.load(Ordering::Relaxed)
    }

    pub fn is_ctrlc_pressed(&self) -> bool {
        self.ctrlc.load(Ordering::SeqCst)
    }

    fn register(&mut self) -> Result<()> {
        let k = self.kill.clone();
        let c = self.ctrlc.clone();

        let mut signals = match Signals::new(&[SIGWINCH, SIGINT, SIGTERM, SIGHUP]){
            Ok(p) => p,
            Err(e) => {
                return Err(e)
                .context("error os signal register");
            },
        };



        Some(thread::spawn(move || {
            for sig in signals.forever() {
                info!("start");

                match sig {
                    SIGINT|SIGTERM|SIGHUP => {
                        if c.load(Ordering::SeqCst) {
                            k.store(true, Ordering::SeqCst);
                            info!("kill notify capture");
                        } else {
                            c.store(true, Ordering::SeqCst);
                            info!("exit notify capture");
                        }            
                    }
                    SIGWINCH => {
                        info!("terminal resized!");
                    }
                    _ => {
                        info!("recv os signal: {:?}", sig);
                    }
                }

                info!("stop");

                thread::sleep(Duration::from_secs(1));
            }
        }));

        Ok(())
    }
}