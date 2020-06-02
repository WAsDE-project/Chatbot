use std::io;
use regex::Regex;
use rand::seq::SliceRandom;

#[derive(Clone, Debug)]
struct Message {
    key: & 'static str,
    matcher: & 'static str,
    values: Vec<& 'static str>,
}

#[no_mangle]
pub extern "C" fn run() {
    let messages = [
        Message {
            key: "generic_messages",
            matcher: r"^$",
            values: vec![
                "Did I already introduce myself? I am Steve.",
                "I'm not marvin.",
                "I am totally not depressed!",
                "Well, hello there!",
            ]
        },
        Message {
            key: "answers",
            matcher: r"\?",
            values: vec![
                "Interesting question, but I am unable to answer that.",
                "I believe I have forgotten the answer to that.",
                "Searching... I could not find the naswer from the database.",
                "Maybe Marvin knows how to answer that question.",
            ],
        }
    ];

    loop {
        let mut input = String::new();
        if let Err(_e) = io::stdin().read_line(&mut input) {
            continue;
        }
        let input = input.trim();
        if input == "exit" {
            break;
        }
        match match_messages(&messages, input) {
            Some(message) => {
                let reply = get_reply(&message);
                println!("{}", reply);
            },
            None => {
                for message in &messages {
                    if message.key == "generic_messages" {
                        let reply = get_reply(&message);
                        println!("{}", reply);
                        break;
                    }
                }
            }
        }
    }
}

fn match_messages(messages: &[Message], input: &str) -> Option<Message> {
    for message in messages {
        let re = Regex::new(message.matcher).unwrap();
        if let Some(value) = re.captures(&input) {
            return Some(message.clone());
        }
    }
    return None;
}

fn get_reply(message: &Message) -> &str {
    return message.values.choose(&mut rand::thread_rng()).unwrap();
}
