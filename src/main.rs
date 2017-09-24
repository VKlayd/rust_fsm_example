extern crate chan;

#[macro_use] mod macro_machine;

use std::thread;

pub enum Message {
    Test {id: usize, cmd: Test::Commands},
    OtherTest {id: usize, cmd: OtherTest::Commands}
}

declare_machine!(
    Test context{id: i16, tx: chan::Sender<Message>}(State1)
    states[State1, State2, State3]
    commands[Cmd1, Cmd2]

    (State1 cont:
        Cmd1 {
            println!("{} {:?} exec Cmd1", context.id, cont);
            if (context.id == 0) {
                context.tx.send(Message::OtherTest{id:0, cmd: OtherTest::Commands::Cmd1});
            }}=> State2;
    )
    (State2 cont:
        Cmd1 {println!("{} {:?} exec Cmd1", context.id, cont);}=> State1;
        Cmd2 {println!("{} {:?} exec Cmd2", context.id, cont);}=> State3;
    )
    (State3 cont:
        Cmd1 {println!("{} {:?} exec Cmd1", context.id, cont);}=> State1;
    )
);

declare_machine!(
    OtherTest context{id: i16, tx: chan::Sender<Message>}(State1)
    states[State1, State2, State3]
    commands[Cmd1, Cmd2]

    (State1 cont:
        Cmd1 {println!("> {} {:?} exec Cmd1", context.id, cont);}=> State2;
    )
    (State2 cont:
        Cmd1 {println!("> {} {:?} exec Cmd1", context.id, cont);}=> State1;
        Cmd2 {println!("> {} {:?} exec Cmd2", context.id, cont);}=> State3;
    )
    (State3 cont:
        Cmd1 {println!("> {} {:?} exec Cmd1", context.id, cont);}=> State1;
    )
);

fn main() {
    let (tx, rx) = chan::sync::<Message>(10);

    let test_machines = vec!(Test::new(0, tx.clone()), Test::new(1, tx.clone()), Test::new(2, tx.clone()));
    let othertest_machines = vec!(OtherTest::new(0, tx.clone()));

    let (tx_fin, rx_fin) = chan::sync(0);
    let mut channels: Vec< chan::Sender<Test::Commands> > = std::vec::Vec::new();
    let mut other_channels: Vec< chan::Sender<OtherTest::Commands> > = std::vec::Vec::new();
    for ma in test_machines {
        let (tx, rx) = chan::sync::<Test::Commands>(10);
        channels.push(tx);
        let tx_fin = tx_fin.clone();
        thread::spawn(move || {
            let mut machine = ma;
            let mut fc = 0;
            loop{
                match rx.recv() {
                    Some(x) => {
                        machine.execute(&x).unwrap();
                        fc += 1;
                        if fc == 10 {
                            tx_fin.send(0);
                        }
                    },
                    _ => ()
                }
            }
        });
    }
    for ma in othertest_machines {
        let (tx, rx) = chan::sync::<OtherTest::Commands>(10);
        other_channels.push(tx);
        let tx_fin = tx_fin.clone();
        thread::spawn(move || {
            let mut machine = ma;
            let mut fc = 0;
            loop{
                match rx.recv() {
                    Some(x) => {
                        machine.execute(&x).unwrap();
                        fc += 1;
                        if fc == 10 {
                            tx_fin.send(0);
                        }
                    },
                    _ => ()
                }
            }
        });
    }
    
    {
        let _a = tx_fin;
    }

    thread::spawn(move || {
        loop {
            match rx.recv() {
                Some(x) => {
                    match x {
                        Message::Test {id, cmd} => {
                            //println!("send to {}", id);
                            channels[id].send(cmd);
                        }
                        Message::OtherTest {id, cmd} => {
                            //println!("send to {}", id);
                            other_channels[id].send(cmd);
                        }
                    }
                },
                _ => ()
            }
        }
    });

    for _ in 0..10 {
        //println!("send");
        tx.send(Message::Test { id: 0, cmd: Test::Commands::Cmd1 });
        tx.send(Message::Test { id: 1, cmd: Test::Commands::Cmd1 });
        //println!("send done");
    }
    //println!("wait all done");
    rx_fin.recv();
    rx_fin.recv();
}
