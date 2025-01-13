use rand::Rng;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone)]
struct Block {
    id: u64,
    validator: String,
    transactions: Vec<String>,
}

#[derive(Debug, Clone)] // Added Clone trait
struct Node {
    name: String,
    stake: u64,
}

impl Node {
    fn new(name: &str, stake: u64) -> Self {
        Node {
            name: name.to_string(),
            stake,
        }
    }
}

struct Blockchain {
    chain: Vec<Block>,
    pending_transactions: Vec<String>,
    nodes: HashMap<String, u64>,
}

impl Blockchain {
    fn new() -> Self {
        Blockchain {
            chain: vec![],
            pending_transactions: vec![],
            nodes: HashMap::new(),
        }
    }

    fn add_transaction(&mut self, transaction: String) {
        self.pending_transactions.push(transaction);
    }

    fn register_node(&mut self, node: Node) {
        self.nodes.insert(node.name, node.stake);
    }

    fn select_validator(&self) -> Option<String> {
        let total_stake: u64 = self.nodes.values().sum();
        if total_stake == 0 {
            return None;
        }

        let mut rng = rand::thread_rng();
        let mut random_weight = rng.gen_range(0..total_stake);

        for (name, stake) in &self.nodes {
            if random_weight < *stake {
                return Some(name.clone());
            }
            random_weight -= stake;
        }

        None
    }

    fn mine_block(&mut self) {
        if let Some(validator) = self.select_validator() {
            let block = Block {
                id: self.chain.len() as u64 + 1,
                validator: validator.clone(),
                transactions: self.pending_transactions.clone(),
            };

            self.chain.push(block);
            self.pending_transactions.clear();
            println!("Block mined by {}", validator);
        } else {
            println!("No validator selected. Check stakes!");
        }
    }

    fn display_chain(&self) {
        for block in &self.chain {
            println!("Block ID: {} | Validator: {} | Transactions: {:?}", block.id, block.validator, block.transactions);
        }
    }
}

fn main() {
    let blockchain = Arc::new(Mutex::new(Blockchain::new()));

    let nodes = vec![
        Node::new("Alice", 50),
        Node::new("Bob", 30),
        Node::new("Charlie", 20),
    ];

    for node in &nodes {
        blockchain.lock().unwrap().register_node(node.clone());
    }

    let blockchain_clone = Arc::clone(&blockchain);
    let miner_thread = thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_secs(3));
            let mut blockchain = blockchain_clone.lock().unwrap();
            blockchain.mine_block();
        }
    });

    blockchain.lock().unwrap().add_transaction("Alice -> Bob: 10 coins".to_string());
    blockchain.lock().unwrap().add_transaction("Bob -> Charlie: 5 coins".to_string());

    thread::sleep(Duration::from_secs(10));
    blockchain.lock().unwrap().display_chain();

    miner_thread.join().unwrap();
}
