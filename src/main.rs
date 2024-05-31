use std::net::TcpStream;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use rand::Rng;
use chrono::prelude::*;

const SPIELER_LATENZ: u64 = 10; // in Sekunden
const NAME_DEFAULT: &str = "SPIELER";

use std::env;

fn main() {
    let mut stream = TcpStream::connect("127.0.0.1:7878").unwrap();
    let args : Vec<String> = env::args().collect();
    let spieler_latenz: u64 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(SPIELER_LATENZ);
    let default_ref = &String::from(NAME_DEFAULT);
    let name = String::from(args.get(1).unwrap_or(default_ref)); // Spielername

    let round = Arc::new(Mutex::new(0));

    loop {
        let mut buffer = [0; 512];
        match stream.read(&mut buffer) {
            Ok(size) => {
                let message = String::from_utf8_lossy(&buffer[..size]);
                println!("Spieler {name} hat eine Nachricht erhalten: {message}");
                if message.trim().contains("STOP") {
                    *round.lock().unwrap() += 1;
                }
                if message.trim().contains("START") {
                    let start_round: DateTime<Local> = Local::now();
                    let mut stream_clone = stream.try_clone().expect("Failed to clone TcpStream");
                    let name_clone = name.clone();
                    let round_start = round.lock().unwrap().clone();
                    let round_ref = Arc::clone(&round);
                    thread::spawn(move || {
                        let latency = rand::thread_rng().gen_range(0..=spieler_latenz);
                        thread::sleep(Duration::from_secs(latency));
                        let actual = *round_ref.lock().unwrap();
                        //println!("{name_clone}: Runde {round_start} erwartet. Tatsächliche Runde: {actual}");
                        if round_start == actual {
                            let wurf: u32 = rand::thread_rng().gen_range(1..=100);
                            let end_round: DateTime<Local> = Local::now();
                            // Senden der Zeiten nur zur Nachvollziehbarkeit in der output-Datei
                            let response = format!("WURF {} {} {} {} {} {} {} {} {} {}",
                                name_clone, wurf,
                                start_round.hour(), start_round.minute(), start_round.second(), start_round.timestamp_subsec_micros(),
                                end_round.hour(), end_round.minute(), end_round.second(), end_round.timestamp_subsec_micros()
                            );
                            stream_clone.write(response.as_bytes()).unwrap();
                        } else {
                            println!("Spieler {name_clone}: Runde {round_start} ist bereits vorbei. Antwort wird nicht geschickt.");
                        }
                    });
                }
                if message.trim().is_empty() {
                    panic!("READ EMPTY MESSAGE, PANICKING!");
                }
            },
            Err(_) => {
                break;
            }
        }
    }
}