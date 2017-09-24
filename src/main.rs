#[macro_use]
extern crate chan;
extern crate threadpool;

#[macro_use] mod macro_machine;

use threadpool::ThreadPool;
use std::thread;
use std::sync::{Arc, Mutex};

declare_machine!(
    Test context{id: i16}(State1)
    states[State1, State2, State3]
    commands[Cmd1, Cmd2]

    (State1 cont:
        Cmd1 {println!("{} {:?} exec Cmd1", context.id, cont);}=> State2;
    )
    (State2 cont:
        Cmd1 {println!("{} {:?} exec Cmd1", context.id, cont);thread::sleep(std::time::Duration::from_secs(1));println!("wait {} done", context.id);}=> State1;
        Cmd2 {println!("{} {:?} exec Cmd2", context.id, cont);}=> State3;
    )
    (State3 cont:
        Cmd1 {println!("{} {:?} exec Cmd1", context.id, cont);}=> State1;
    )
);

enum Message {
    Test {id: usize, cmd: Test::Commands}
}

fn main() {
    let mut test_machines = vec!(Test::new(0), Test::new(1), Test::new(2));
    let n_workers = 2;
    let pool = ThreadPool::new(n_workers);

    let (tx, rx) = chan::sync::<Message>(6);

    let mut channels: Vec< chan::Sender<Test::Commands> > = std::vec::Vec::new();
    for ma in test_machines {
        let (tx, rx) = chan::sync::<Test::Commands>(6);
        channels.push(tx);
        thread::spawn(move || {
            let mut machine = ma;
            loop{
                match rx.recv() {
                    Some(x) => {
                        machine.execute(&x).unwrap();
                    },
                    _ => ()
                }
            }
        });
    }

    thread::spawn(move || {
        loop {
            match rx.recv() {
                Some(x) => {
                    match x {
                        Message::Test {id, cmd} => {
                            println!("send to {}", id);
                            channels[id].send(cmd);
                        }
                    }
                },
                _ => ()
            }
        }
    });

    tx.send(Message::Test {id:0, cmd: Test::Commands::Cmd1});
    tx.send(Message::Test {id:0, cmd: Test::Commands::Cmd1});
    tx.send(Message::Test {id:0, cmd: Test::Commands::Cmd1});
    tx.send(Message::Test {id:0, cmd: Test::Commands::Cmd1});
    tx.send(Message::Test {id:1, cmd: Test::Commands::Cmd1});
    tx.send(Message::Test {id:1, cmd: Test::Commands::Cmd2});
    tx.send(Message::Test {id:1, cmd: Test::Commands::Cmd1});

    thread::sleep(std::time::Duration::from_secs(5));
}
