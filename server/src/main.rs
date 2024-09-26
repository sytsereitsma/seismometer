use serialport;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::mpsc;

struct Record {
    timestamp: u64,
    x: u32,
    y: u32,
    z: u32,
    x_filt: u32,
    y_filt: u32,
    z_filt: u32,
}

struct DataProcessor {
    data: Mutex<Vec<u8>>,
    detection_window: Vec<u8>,
    window_index: usize,
}

impl DataProcessor {
    fn new() -> DataProcessor {
        DataProcessor {
            data: Mutex::new(Vec::new()),
            detection_window: vec![0; 1024],
            window_index: 0,
        }
    }

    fn processing_thread(proc: Arc<DataProcessor>, stop_thread: Arc<AtomicBool>) {
        loop {
            if stop_thread.load(Ordering::Relaxed) {
                break;
            }

            let mut data = proc.data.lock().unwrap();                        
            let newline = data.iter().position(|&x| x == b'\n');

            if let Some(pos) = newline {
                proc.window_index += 1;
                if (proc.window_index == 500)
                {
                    proc.window_index = 0;
                    println!("Window detected");
                }
                //let line = &data[..pos];
                // proc.process(line);
                data.drain(..pos+1);
            }            
        }
    }

    fn process(&self, new_data: &[u8]) {
        let mut data = self.data.lock().unwrap();
        data.extend(new_data);
    }


    // fn detect(&self) {
    //     // Implement your detection algorithm here
    // }
}


fn main() {
    let stop_thread = Arc::new(AtomicBool::new(false));

    // let mut file = OpenOptions::new()
    //     .create(true)
    //     .write(true)
    //     .append(true)
    //     .open("seismodata.txt")
    //     .unwrap();

    let _ = serialport::new("COM3", 115_200)
        .timeout(Duration::from_millis(1000))
        .open()
        .and_then(|mut port| {
            let (tx, rx) = mpsc::channel();

            let data_processor = Arc::new(DataProcessor::new());
            
            let data_processor_thread = {
                let stop_processor = stop_thread.clone();
                let processor_data = data_processor.clone();
                thread::spawn(move || DataProcessor::processing_thread(processor_data, stop_processor))    
            };
            
            let read_thread = {
                let mut buf: Vec<u8> = vec![0; 1024];
            
                let stop_read = stop_thread.clone();
                let rd_processor = data_processor.clone();
                
                thread::spawn(move || loop {
                    if stop_read.load(Ordering::Relaxed) {
                        break;
                    }

                    match port.read(&mut buf) {
                        Ok(n) => {
                            // file.write_all(&buf[..n]).unwrap();
                            //# println!("{}", String::from_utf8((&buf[..n]).to_vec()).unwrap());
                            rd_processor.process(&buf[..n]);
                        }
                        Err(e) => {
                            println!("Error: {:?}", e);
                        }
                    }
                })
            };

            Ok((read_thread, data_processor_thread))
        })
        .or_else(|e| {
            println!("Read thread error: {:?}", e);
            Err(e)
        })
        .and_then(|(read_thread,data_processor_thread)| {
            let stop = stop_thread.clone();

            let _ = ctrlc::set_handler(move || {
                stop.store(true, Ordering::Relaxed);
            });

            read_thread.join().unwrap();
            data_processor_thread.join().unwrap();

            Ok(())
        });
}
