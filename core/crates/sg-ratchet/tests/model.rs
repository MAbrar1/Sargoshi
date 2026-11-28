//! A randomized network model: both parties send, and messages are delivered late, out of order,
//! never, or twice. Every delivered message must decrypt to what was sent, every duplicate must
//! be refused, and a refusal must never change a session.
#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "test model: a failed unwrap or index is the intended test failure"
)]

mod common;

use common::pair;
use proptest::prelude::*;
use sg_ratchet::Session;

#[derive(Debug, Clone)]
enum Op {
    Send(bool),
    Deliver(bool, usize),
    Drop(bool, usize),
    Replay(bool, usize),
}

fn op() -> impl Strategy<Value = Op> {
    prop_oneof![
        3 => any::<bool>().prop_map(Op::Send),
        4 => (any::<bool>(), any::<usize>()).prop_map(|(a, i)| Op::Deliver(a, i)),
        1 => (any::<bool>(), any::<usize>()).prop_map(|(a, i)| Op::Drop(a, i)),
        1 => (any::<bool>(), any::<usize>()).prop_map(|(a, i)| Op::Replay(a, i)),
    ]
}

struct Network {
    alice: Session,
    bob: Session,
    to_alice: Vec<(Vec<u8>, String)>,
    to_bob: Vec<(Vec<u8>, String)>,
    delivered_to_alice: Vec<Vec<u8>>,
    delivered_to_bob: Vec<Vec<u8>>,
    counter: u32,
}

impl Network {
    fn new() -> Self {
        let (alice, bob) = pair();
        Self {
            alice,
            bob,
            to_alice: Vec::new(),
            to_bob: Vec::new(),
            delivered_to_alice: Vec::new(),
            delivered_to_bob: Vec::new(),
            counter: 0,
        }
    }

    fn step(&mut self, op: &Op) -> Result<(), TestCaseError> {
        match *op {
            Op::Send(from_alice) => {
                let (sender, queue) = if from_alice {
                    (&mut self.alice, &mut self.to_bob)
                } else {
                    (&mut self.bob, &mut self.to_alice)
                };
                if sender.can_send() {
                    self.counter += 1;
                    let text = format!("message {}", self.counter);
                    let bytes = sender.encrypt(text.as_bytes()).unwrap();
                    queue.push((bytes, text));
                }
            }
            Op::Deliver(to_alice, i) => {
                let (receiver, queue, log) = if to_alice {
                    (
                        &mut self.alice,
                        &mut self.to_alice,
                        &mut self.delivered_to_alice,
                    )
                } else {
                    (&mut self.bob, &mut self.to_bob, &mut self.delivered_to_bob)
                };
                if !queue.is_empty() {
                    let (bytes, text) = queue.remove(i % queue.len());
                    let plain = receiver.decrypt(&bytes).unwrap();
                    prop_assert_eq!(plain.as_slice(), text.as_bytes());
                    log.push(bytes);
                }
            }
            Op::Drop(to_alice, i) => {
                let queue = if to_alice {
                    &mut self.to_alice
                } else {
                    &mut self.to_bob
                };
                if !queue.is_empty() {
                    queue.remove(i % queue.len());
                }
            }
            Op::Replay(to_alice, i) => {
                let (receiver, log) = if to_alice {
                    (&mut self.alice, &self.delivered_to_alice)
                } else {
                    (&mut self.bob, &self.delivered_to_bob)
                };
                if !log.is_empty() {
                    let before = receiver.to_bytes().unwrap().to_vec();
                    prop_assert!(
                        receiver.decrypt(&log[i % log.len()]).is_err(),
                        "a replay was accepted"
                    );
                    prop_assert_eq!(receiver.to_bytes().unwrap().to_vec(), before);
                }
            }
        }
        Ok(())
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(48))]

    #[test]
    fn any_schedule_delivers_exactly_what_was_sent(ops in proptest::collection::vec(op(), 1..70)) {
        let mut net = Network::new();
        for op in &ops {
            net.step(op)?;
        }
        // Drain everything still in flight, then check the sessions are still fully usable.
        while !net.to_bob.is_empty() {
            net.step(&Op::Deliver(false, 0))?;
        }
        while !net.to_alice.is_empty() {
            net.step(&Op::Deliver(true, 0))?;
        }
        net.step(&Op::Send(true))?;
        net.step(&Op::Deliver(false, 0))?;
        net.step(&Op::Send(false))?;
        net.step(&Op::Deliver(true, 0))?;
    }
}
