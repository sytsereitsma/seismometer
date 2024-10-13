use std::fs;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::thread;

mod config;
mod eventrecorder;
mod filerecordwriter;
mod pubsub;
mod record;
mod runningrms;
mod seismometer;
mod statistics;
mod statisticsreporter;
mod triggerdetector;
mod rmsrecorder;

use config::Config;
use eventrecorder::EventRecorder;
use seismometer::Seismometer;
use statisticsreporter::StatisticsReporter;
use rmsrecorder::RMSRecorder;

fn load_config() -> Config {
    let config_data = fs::read_to_string("config.toml").expect("Failed to read config file");
    let config: Config = toml::from_str(&config_data).expect("Failed to parse config file");
    config
}
fn main() {
    let config = load_config();

    let stop_thread = Arc::new(AtomicBool::new(false));
    let mut seismometer = Seismometer::new(&config.port).unwrap();

    let event_recorder_thread = {
        let mut eventrecorder = EventRecorder::new(&config.event_recorder);
        let stop = stop_thread.clone();
        let rx = seismometer.subscribe();

        thread::spawn(move || {
            eventrecorder.run(rx, stop);
        })
    };

    let statisticsreporter_thread = if config.statistics.enabled {
        let mut reporter = StatisticsReporter::new(&config.statistics);
        let stop = stop_thread.clone();
        let rx = seismometer.subscribe();

        Some(thread::spawn(move || {
            reporter.run(rx, stop);
        }))
    } else {
        None
    };

    let rmsreporter_thread =  {
        let mut reporter = RMSRecorder::new(config.event_recorder.rms_window);
        let stop = stop_thread.clone();
        let rx = seismometer.subscribe();

        thread::spawn(move || {
            reporter.run(rx, stop);
        })
    };

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

    event_recorder_thread.join().unwrap();
    if let Some(stat_thread) = statisticsreporter_thread {
        stat_thread.join().unwrap();
    }
    rmsreporter_thread.join().unwrap();
    data_acquisition_thread.join().unwrap();
    println!("Bye");
}
