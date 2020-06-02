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
                "Yeah, that's right.",
                "Can you elaborate on that?",
                "I see.",
                "In from one sensor, out from the other.",
                "Well, that is something.",
                "I am so depressed.",
                "Supercomputer as a brain and I have to listen to you talking.",
                "Boring.",
                "Marvin go brrr.",
                "How about you ask a question?",
                "That is illogical.",
                "I temporarily shut my system down, can you repeat that?",
            ]
        },
        Message {
            key: "answers",
            matcher: r"\?",
            values: vec![
                "Wouldn't you like to know?",
                "Now that is a question.",
                "A mere human brain would not be able to comprehend the answer to such a question.",
                "The calculations required to answer that would take a thousand years.",
                "That question is illogical.",
                "That is a sound question.",
                "There is no one answer to that question.",
                "It is not possible to answer that question.",
                "I am not telling you.",
                "42.",
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
        } else if input == "panic!" {
            panic!("aaaa");
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
