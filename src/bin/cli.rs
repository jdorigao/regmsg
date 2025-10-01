use std::env;
use serde_json::json;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Collect CLI args: first arg is command, rest are args
    let mut args = env::args().skip(1); // skip binary name
    let cmd = match args.next() {
        Some(c) => c,
        None => {
            eprintln!("Usage: regmsg <command> [args...]");
            std::process::exit(1);
        }
    };
    let extra_args: Vec<String> = args.collect();

    // Prepare request
    let request = json!({
        "cmd": cmd,
        "args": extra_args,
    });

    // Connect to daemon via ZeroMQ
    let ctx = zmq::Context::new();
    let socket = ctx.socket(zmq::REQ)?;
    socket.connect("ipc:///tmp/regmsg.sock")?;

    // Send request
    socket.send(serde_json::to_vec(&request)?, 0)?;

    // Receive reply
    let reply_bytes = socket.recv_bytes(0)?;
    let reply: serde_json::Value = serde_json::from_slice(&reply_bytes)?;

    // Print nicely (stdout for piping / legacy scripts)
    if reply.is_object() || reply.is_array() {
        println!("{}", serde_json::to_string_pretty(&reply)?);
    } else {
        println!("{}", reply);
    }

    Ok(())
}
