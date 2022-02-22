use std::process::{Command, Stdio};

use regex::Regex;
use serial_test::serial;

#[test]
#[serial]
fn it_works() {
    // Start up the server
    let mut server = Command::new("cargo")
        .args(["run", "--release"])
        .spawn()
        .expect("Failed to run the server!");

    let response = reqwest::blocking::get("http://127.0.0.1:3000").unwrap();

    assert_eq!(response.status(), 200);

    let is_valid_html = Regex::new("<(“[^”]*”|'[^’]*’|[^'”>])*>")
        .unwrap()
        .is_match(&response.text().unwrap());

    assert!(is_valid_html);

    server.kill().expect("Failed killing the server!");
}

#[test]
#[serial]
fn it_works_in_parallel() {
    // Start up the server
    let mut server = Command::new("cargo")
        .args(["run", "--release"])
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to run the server!");

    for _ in 0..100 {
        reqwest::blocking::get("http://127.0.0.1:3000").unwrap();
    }

    server.kill().expect("Failed killing the server!");
    let output = server.wait_with_output().unwrap();

    let output = String::from_utf8(output.stdout).unwrap();

    let mut threads_ids: Vec<i32> = output
        .split("\nexecuting on thread: ")
        .map(|s| s.trim().parse())
        .filter(|r| r.is_ok())
        .map(|r| r.unwrap())
        .collect();

    threads_ids.sort();
    threads_ids.dedup();

    assert!(
        threads_ids.len() >= 3,
        "Threads don't behave as intended (not parallel)!"
    );
}
