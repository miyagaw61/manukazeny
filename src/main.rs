extern crate clap;
extern crate colored;
extern crate regex;
extern crate serde_json;
#[macro_use]
extern crate chan;
extern crate chan_signal;
#[macro_use]
extern crate lazy_static;

use std::fs::OpenOptions;
use std::io::Read;
use std::process::Command;
use colored::*;
use serde_json::Value;
use std::thread;
use chan_signal::Signal;
use std::sync::RwLock;
use clap::{App, Arg, SubCommand};

struct SystemResult {
    stdout: String,
    stderr: String,
    status: i32
}

impl SystemResult {
    fn new(output: std::process::Output) -> SystemResult {
        let mut stdout: Vec<char> = std::str::from_utf8(&output.stdout[..]).unwrap().to_string().chars().collect();
        stdout.pop();
        let stdout: String = stdout.into_iter().collect();
        let mut stderr: Vec<char> = std::str::from_utf8(&output.stderr[..]).unwrap().to_string().chars().collect();
        stderr.pop();
        let stderr: String = stderr.into_iter().collect();
        let mut result = SystemResult {
            stdout: stdout,
            stderr: stderr,
            status: 0
        };
        if result.stderr.chars().count() > 0 {
            result.status = 1
        }
        result
    }
}

fn system(command: &str) -> SystemResult {
    let result = Command::new("sh")
        .arg("-c")
        .arg(command)
        .output()
        .expect("failed to execute process");
    let result = SystemResult::new(result);
    if result.status != 0 {
        let emsg = [
            "== ".red().to_string(),
            "[+]ERROR".red().bold().to_string(),
            " =====================".red().to_string()
        ].join("");
        println!("{}", emsg);
        println!("{}", result.stderr);
        println!("{}", "=================================".red().to_string());
    }
    result
}

fn system_allow_stderr(command: &str) -> SystemResult {
    let result = Command::new("sh")
        .arg("-c")
        .arg(command)
        .output()
        .expect("failed to execute process");
    SystemResult::new(result)
}

fn process(command: &str) -> std::process::ExitStatus {
    let mut child = Command::new("sh")
        .arg("-c")
        .arg(command)
        .spawn()
        .expect("failed to execute process");
    child.wait().unwrap()
}

fn mining(json_data: serde_json::Value) {
    let mut addresses_counter = 0;
    loop {
        {
            let mut address = ADDRESS.write().unwrap();
            *address = json_data["addresses"][addresses_counter].to_string();
            if (address.len() < 5) {
                std::process::exit(1);
            }
            process([
                    "echo ",
                    address.as_str(),
                    " > /tmp/address.manukazeny"
            ].join("").as_str());
            addresses_counter += 1;
            send_slack(["process start: ", &*address].join("").as_str());
            process([
                    "minerd -a yescrypt -o stratum+tcp://bitzeny.bluepool.info:3330 -r 3 -u ",
                    &*address.as_str(),
                    " 1> /dev/null 2> manukazeny.log"
            ].join("").as_str());
        }
        //let mut newest = "".to_string(); //保存用にloopから出した
        //loop {
        //    let mut data = String::new();
        //    let file_name = "manukazeny.log";
        //    let mut f = match OpenOptions::new().read(true).open(file_name) {
        //        Ok(f) => f,
        //        Err(_) => { continue }
        //    };
        //    f.read_to_string(&mut data).expect(["Can not read file: ", file_name].join("").as_str());
        //    let data_vec: Vec<&str> = data.split('\n').collect();
        //    if data_vec.len() <= 1 { continue }
        //    let now = data_vec[data_vec.len()-2].to_string(); //data_vecの最後に""が入ってしまうため-1ではなく-2
        //    if newest != now {
        //        println!("{}", now);
        //        newest = now;
        //}
    }
}

//fn mining_wrap(data: serde_json::Value) { //並列化,シグナル処理
//    let signal = chan_signal::notify(&[Signal::INT]);
//    let (sdone, rdone) = chan::sync(0);
//    thread::spawn(move || mining(sdone, data));
//    chan_select! {
//        signal.recv() => {
//            let address = ADDRESS.read().unwrap();
//            let sum = SUM.read().unwrap();
//            let loop_counter = LOOP_COUNTER.read().unwrap();
//            let khash_rate: f64 = *sum / *loop_counter;
//            send_slack(["process dead: ", &*address, "\nkhash-rate: ", khash_rate.to_string().as_str()].join("").as_str());
//            process("kill -9 $(pgrep -n minerd)");
//            process("rm -rf manukazeny.log"); //必要に応じて。
//            std::process::exit(0);
//        },
//        rdone.recv() => {}
//    }
//}

fn send_slack(message: &str) {
    let before = [r#"curl -s -X POST --data-urlencode "payload={\"channel\": \""#, std::env::var("RUSGIT_SLACK_CHANNEL").expect("[!]Please export RUSGIT_SLACK_CHANNEL.").as_str(), r#"\", \"username\": \"Manuka Zeny\", \"text\": \""#].join("");
    let before = before.as_str();
    let after = [r#"\", \"icon_emoji\": \":ghost:\"}" "#, std::env::var("RUSGIT_SLACK_URL").expect("[!]Please export RUSGIT_SLACK_URL.").as_str(), r#" > /dev/null"#].join("");
    let after = after.as_str();
    let cmd = [before, message, after].join("");
    process(&cmd);
}

lazy_static! {
    static ref ADDRESS: RwLock<String> = RwLock::new(String::new());
    static ref SUM: RwLock<f64> = RwLock::new(0.00);
    static ref LOOP_COUNTER: RwLock<f64> = RwLock::new(1.00);
}

fn start(matches: &clap::ArgMatches) {
    let mut data = String::new();
    let file_name = matches.subcommand_matches("start").unwrap().value_of("json_file").unwrap();
    let mut f = match OpenOptions::new().read(true).open(file_name) {
        Ok(f) => f,
        Err(_) => {
            print!("Can not open file: ");
            println!("{}", file_name);
            std::process::exit(0);
        }
    };
    f.read_to_string(&mut data).expect(["Can not read file: ", file_name].join("").as_str());
    let data: Value = serde_json::from_str(&data[..]).expect("Can not deserialize");
    mining(data);
}

fn help() {
    println!("\
USAGE:
    manukazeny [SUBCOMMAND]
manukazeny -h for help\
");
}

fn stop() {
    let mut address = String::new();
    let file_name = "/tmp/address.manukazeny";
    let mut f = match OpenOptions::new().read(true).open(file_name) {
        Ok(f) => f,
        Err(_) => {
            print!("Can not open file: ");
            println!("{}", file_name);
            std::process::exit(0);
        }
    };
    f.read_to_string(&mut address).expect(["Can not read file: ", file_name].join("").as_str());
    let mut data = String::new();
    let file_name = "./manukazeny.log";
    let mut f = match OpenOptions::new().read(true).open(file_name) {
        Ok(f) => f,
        Err(_) => {
            print!("Can not open file: ");
            println!("{}", file_name);
            std::process::exit(0);
        }
    };
    f.read_to_string(&mut data).expect(["Can not read file: ", file_name].join("").as_str());
    let data_vec: Vec<&str> = data.split("\n").collect();
    {
        let mut loop_counter = LOOP_COUNTER.write().unwrap();
        *loop_counter = 0.00;
    }
    for data in data_vec {
        if data.contains("khash/s (yay!!!)") {
            let khash_index = data.find("khash/s (yay!!!)").unwrap_or(0);
            if khash_index != 0 {
                let khash_rate = &data[khash_index-5..khash_index-1];
                let khash_rate: f64 = khash_rate.parse().unwrap();
                println!("khash_rate: {}", khash_rate);
                {
                    let mut sum = SUM.write().unwrap();
                    *sum += khash_rate;
                }
                {
                    let mut loop_counter = LOOP_COUNTER.write().unwrap();
                    *loop_counter += 1.00;
                }
            }
        }
    }
    {
        let sum = SUM.read().unwrap();
        let loop_counter = LOOP_COUNTER.read().unwrap();
        println!("{}", *loop_counter);
        let khash_rate: f64 = *sum / *loop_counter;
        send_slack(["process dead: ", address.as_str(), "\nkhash-rate: ", khash_rate.to_string().as_str()].join("").as_str());
    }
    process("pkill -9 manukazeny; pkill -9 minerd");
    process("rm -rf manukazeny.log"); //必要に応じて。
    std::process::exit(0);
}

fn main() {
    let matches = App::new("Manuka Zeny")
        .version("0.0.1")
        .author("miyagaw61 <miyagaw61@gmail.com>")
        .about("Cpuminer Wrapper in Rust")
        .subcommand(SubCommand::with_name("start")
                    .about("start manukazeny")
                    .arg(Arg::with_name("json_file")
                         .help(r#"config json file
              (Example)
              { "address": ["ABC", "IJK", "XYZ"] }"#)
                         .takes_value(true)
                         .required(true)
                         )
                    )
        .subcommand(SubCommand::with_name("stop")
                    .about("stop manukazeny")
                    )
        .get_matches();
    let sub_command = matches.subcommand_name().unwrap_or("");
    match sub_command {
        "start" => start(&matches),
        "stop" => stop(),
        _ => help()
    }
}
