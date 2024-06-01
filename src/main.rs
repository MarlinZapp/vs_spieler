use std::net::TcpStream;
use std::io::{Read, Write};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use rand::Rng;
use chrono::prelude::*;
use std::cmp;

/*
Erstes Argument: Spielername ohne Leerzeichen
Zweites Argument: Spielerlatenz (default: 6 Sekunden)
Drittes Argument: IP and Port (default: 127.0.0.1:7878)
*/

const SPIELER_LATENZ: u64 = 8; // in Sekunden
const IP_AND_PORT : &str = "127.0.0.1:7878";

use std::env;

fn main() {
    let args : Vec<String> = env::args().collect();
    let name = String::from(args.get(1).expect("Erwartet Spielernamen als erstes Argument!"));
    let spieler_latenz: u64 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(SPIELER_LATENZ);
    let ip_and_port: &str = args.get(3).map(|s| s.as_str()).unwrap_or(IP_AND_PORT);

    let mut stream = TcpStream::connect(ip_and_port).unwrap();

    let lc = Arc::new(AtomicU32::new(0));
    let round = Arc::new(AtomicU32::new(0));
    let mut buffer = [0; 512];
    stream.read(&mut buffer).expect("WELCOME-Nachricht nicht erfolgreich gelesen!");
    
    // Find the position of the first null byte
    let null_pos = buffer.iter().position(|&x| x == 0).unwrap_or(buffer.len());
    // Convert the buffer up to the first null byte to a string
    let message = String::from_utf8_lossy(&buffer[..null_pos]);

    let parts : Vec<&str> = message.trim().split_whitespace().collect();
    let mut teilaufgabe = 'B';
    if parts[1] == "A" {
        teilaufgabe = 'A';
    }
    println!("Client verbunden mit {ip_and_port} für Teilaufgabe {teilaufgabe}...");

    loop {
        let mut buffer = [0; 512];
        match stream.read(&mut buffer) {
            Ok(size) => {
                let message = String::from_utf8_lossy(&buffer[..size]);
                println!("Spieler {name} hat eine Nachricht erhalten: {message}");
                if message.trim().contains("STOP") {
                    round.fetch_add(1, Ordering::SeqCst);
                    if teilaufgabe == 'B' {
                        let lc_server = parse_lc_server_from_message(&message);
                        // max(lc, client_lc)
                        lc.fetch_update(Ordering::SeqCst, Ordering::SeqCst, |current|
                            Some(cmp::max(lc_server, current))
                        ).expect("Lamport-Zeit konnte nicht geupdated werden.");
                        // lc++
                        lc.fetch_add(1, Ordering::SeqCst);
                    }
                }
                if message.trim().contains("START") {
                    if teilaufgabe == 'A' {
                        respond(teilaufgabe, &stream, &name, &round, &lc, spieler_latenz);
                    } else {
                        let lc_server = parse_lc_server_from_message(&message);
                        
                        // max(lc, client_lc)
                        lc.fetch_update(Ordering::SeqCst, Ordering::SeqCst, |current|
                            Some(cmp::max(lc_server, current))
                        ).expect("Lamport-Zeit konnte nicht geupdated werden.");
                        // lc++
                        lc.fetch_add(1, Ordering::SeqCst);

                        respond(teilaufgabe, &stream, &name, &round, &lc, spieler_latenz);
                    }
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

fn parse_lc_server_from_message(message: &std::borrow::Cow<str>) -> u32 {
    // parse server lamport time
    let parts : Vec<&str> = message.trim().split_whitespace().collect();
    let mut lc_server = "";
    if parts.len() == 2 {
        lc_server = parts[1];
    } else if parts.len() == 3 && parts[0] == "STOP" {
        lc_server = parts[2];
    } else {
        println!("Lamport-Zeit kann nicht aus der Nachricht {message} geparsed werden!");
    }
    let lc_server: u32 = lc_server.parse().expect(format!("Expected Lamport-Zeit in {lc_server}").as_str());
    lc_server
}

fn respond(teilaufgabe : char, stream : &TcpStream, name : &String, round : &Arc<AtomicU32>, lc : &Arc<AtomicU32> , spieler_latenz :u64 ) {
    let start_round: DateTime<Local> = Local::now();
    let mut stream_clone = stream.try_clone().expect("Failed to clone TcpStream");
    let name_clone = name.clone();
    let round_start = round.load(Ordering::SeqCst);
    let round_ref = Arc::clone(&round);
    let lc_ref = Arc::clone(&lc);
    thread::spawn(move || {
        let latency = rand::thread_rng().gen_range(0..=spieler_latenz);
        thread::sleep(Duration::from_secs(latency));
        let actual = round_ref.load(Ordering::SeqCst);
        //println!("{name_clone}: Runde {round_start} erwartet. Tatsächliche Runde: {actual}");
        if round_start == actual {
            let wurf: u32 = rand::thread_rng().gen_range(1..=100);
            let end_round: DateTime<Local> = Local::now();
            if teilaufgabe == 'A' {
                // Senden der Zeiten nur zur Nachvollziehbarkeit in der output-Datei
                let response = format!("WURF {} {} {} {} {} {} {} {} {} {}",
                    name_clone, wurf,
                    start_round.hour(), start_round.minute(), start_round.second(), start_round.timestamp_subsec_micros(),
                    end_round.hour(), end_round.minute(), end_round.second(), end_round.timestamp_subsec_micros());
                stream_clone.write(response.as_bytes()).unwrap();
            } else {
                // Senden der Zeiten nur zur Nachvollziehbarkeit in der output-Datei
                let response = format!("WURF {} {} {} {} {} {} {} {} {} {} {}",
                    name_clone, wurf, lc_ref.load(Ordering::SeqCst),
                    start_round.hour(), start_round.minute(), start_round.second(), start_round.timestamp_subsec_micros(),
                    end_round.hour(), end_round.minute(), end_round.second(), end_round.timestamp_subsec_micros());
                stream_clone.write(response.as_bytes()).unwrap();
            }
        } else {
            println!("Spieler {name_clone}: Runde {round_start} ist bereits vorbei. Antwort wird nicht geschickt.");
        }
    });
}