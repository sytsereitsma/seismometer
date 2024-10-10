use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::thread;
use std::fs;

mod config;
mod eventrecorder;
mod filerecordwriter;
mod pubsub;
mod record;
mod seismometer;
mod statistics;
mod triggerdetector;
mod statisticsreporter;

use clap;
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

    let matches = clap::command!()
        .arg(clap::arg!(statistics: -s).action(clap::ArgAction::SetTrue))
        .get_matches();

    let stop_thread = Arc::new(AtomicBool::new(false));
    let mut seismometer = Seismometer::new(&config.port).unwrap();

    let event_recorder_thread = {
        let mut eventrecorder = EventRecorder::new(&config);
        let stop = stop_thread.clone();
        let rx = seismometer.subscribe();

        thread::spawn(move || {
            eventrecorder.run(rx, stop);
        })
    };

    let statisticsreporter_thread = {
        let mut reporter = StatisticsReporter::new();
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
    statisticsreporter_thread.join().unwrap();
    data_acquisition_thread.join().unwrap();
    println!("Bye");
}
