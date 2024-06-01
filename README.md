# Programm starten

## Erst Spielleiter starten aus folgendem Repository:
https://github.com/MarlinZapp/vs_spielleiter

## Dann entsprechende Anzahl an Spielern starten:

- Erstes Argument: Spielername ohne Leerzeichen
- Zweites Argument: Spielerlatenz (default: 6 Sekunden)
- Drittes Argument: IP and Port (default: 127.0.0.1:7878)

Wenn rust und cargo schon installiert sind, z.B.:
`cargo run -- pippi_langstrumpf 15 127.0.0.1:8888`

Wenn nicht:
`./target/release/spieler pippi_langstrumpf 15 127.0.0.1:8888`
