use std::fs;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::thread;

mod config;
mod emafilter;
mod eventrecorder;
mod filerecordwriter;
mod pubsub;
mod record;
mod recordhandler;
mod seismometer;
mod statistics;
mod statisticsreporter;
mod triggerdetector;

use crate::recordhandler::RecordHandler;
use config::Config;
use eventrecorder::EventRecorder;
use seismometer::Seismometer;
use statisticsreporter::StatisticsReporter;

fn load_config() -> Config {
    let config_data = fs::read_to_string("config.toml").expect("Failed to read config file");
    let config: Config = toml::from_str(&config_data).expect("Failed to parse config file");
    config
}

fn main() {
    let config = load_config();

    let stop_thread = Arc::new(AtomicBool::new(false));
    let mut seismometer = Seismometer::new(&config.port).unwrap();

    let mut handler_threads = vec![];

    handler_threads.push({
        let mut eventrecorder = EventRecorder::new(&config.event_recorder);
        let stop = stop_thread.clone();
        let rx = seismometer.subscribe();

        thread::spawn(move || {
            eventrecorder.run(rx, stop);
        })
    });

    if config.statistics.enabled {
        handler_threads.push({
            let mut reporter = StatisticsReporter::new(&config.statistics);
            let stop = stop_thread.clone();
            let rx = seismometer.subscribe();

            thread::spawn(move || {
                reporter.run(rx, stop);
            })
        });
    }

    if config.raw_data_recorder.enabled {
        handler_threads.push({
            let mut reporter =
                filerecordwriter::FileRecordWriter::new(&config.raw_data_recorder.filename)
                    .unwrap();
            let stop = stop_thread.clone();
            let rx = seismometer.subscribe();

            thread::spawn(move || {
                reporter.run(rx, stop);
            })
        });
    }

    // The main thread is responsible for running the seismometer
    let data_acquisition_thread = {
        let stop = stop_thread.clone();

        thread::spawn(move || {
            seismometer.run(stop);
        })
    };

    // Everything is up and running, now wait for a CTRL+C
    let stop = stop_thread.clone();

    let _ = ctrlc::set_handler(move || {
        stop.store(true, Ordering::Relaxed);
    });

    for t in handler_threads {
        t.join().unwrap();
    }

    data_acquisition_thread.join().unwrap();

    println!("Bye");
}
