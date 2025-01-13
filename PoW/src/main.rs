use chrono::{Utc};
use rand::Rng;
use sha2::{Sha256, Digest}; // sha2 crate'i içe aktarıyoruz
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone)]
struct Block {
    id: u64,
    timestamp: String,
    previous_hash: String,
    transactions: Vec<String>,
    nonce: u64,
    hash: String,
    validator: String,
    difficulty: usize,
}

impl Block {
    fn new(id: u64, previous_hash: String, transactions: Vec<String>, validator: String, difficulty: usize) -> Self {
        let timestamp = Utc::now().to_rfc3339();
        Block {
            id,
            timestamp,
            previous_hash,
            transactions,
            nonce: 0,
            hash: String::new(),
            validator,
            difficulty,
        }
    }

    fn calculate_hash(&self) -> String {
        let data = format!(
            "{}{}{}{}{}{}",
            self.id, self.timestamp, self.previous_hash, self.nonce, self.validator, self.transactions.join("")
        );
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }

    fn mine_block(&mut self) {
        let target = vec!['0'; self.difficulty].iter().collect::<String>();
        
        while &self.calculate_hash()[..self.difficulty] != target {
            self.nonce += 1;
            self.hash = self.calculate_hash();
        }
    }

    fn finalize_block(&mut self) {
        self.hash = self.calculate_hash();
    }
}

#[derive(Debug, Clone)]
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
    chain: VecDeque<Block>,
    pending_transactions: Vec<String>,
    nodes: HashMap<String, u64>,
    difficulty: usize, 
}

impl Blockchain {
    fn new(difficulty: usize) -> Self {
        let mut blockchain = Blockchain {
            chain: VecDeque::new(),
            pending_transactions: vec![],
            nodes: HashMap::new(),
            difficulty,
        };
        let genesis_block = Block::new(0, String::from("0"), vec!["Genesis Block".to_string()], "System".to_string(), difficulty);
        blockchain.add_block_to_chain(genesis_block);
        blockchain
    }

    fn add_transaction(&mut self, transaction: String) {
        self.pending_transactions.push(transaction);
    }

    fn register_node(&mut self, node: Node) {
        self.nodes.insert(node.name.clone(), node.stake);
    }

    fn finalize_pending_transactions(&mut self) {
        let last_block = self.chain.back().expect("Blockchain is empty; no last block found.");
        let mut new_block = Block::new(
            last_block.id + 1,
            last_block.hash.clone(),
            self.pending_transactions.clone(),
            "Validator".to_string(),
            self.difficulty,
        );
        new_block.mine_block();
        new_block.finalize_block();
        self.add_block_to_chain(new_block);
        self.pending_transactions.clear();
    }

    fn add_block_to_chain(&mut self, block: Block) {
        self.chain.push_back(block);
    }

    fn print_chain(&self) {
        for block in &self.chain {
            println!("{:?}", block);
        }
    }
}

fn main() {
    let blockchain = Arc::new(Mutex::new(Blockchain::new(3)));

    let nodes = vec![
        Node::new("Alice", 50),
        Node::new("Bob", 30),
        Node::new("Charlie", 20),
    ];

    for node in &nodes {
        blockchain.lock().expect("Failed to acquire lock on blockchain.").register_node(node.clone());
    }

    let blockchain_clone = Arc::clone(&blockchain);
    let validator_thread = thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_secs(5));
            if let Ok(mut blockchain) = blockchain_clone.lock() {
                blockchain.finalize_pending_transactions();
            } else {
                println!("Failed to acquire lock for validator thread.");
            }
        }
    });

    {
        let mut blockchain = blockchain.lock().expect("Failed to acquire lock on blockchain for transactions.");
        blockchain.add_transaction("Alice -> Bob: 10 coins".to_string());
        blockchain.add_transaction("Bob -> Charlie: 5 coins".to_string());
        blockchain.add_transaction("Charlie -> Alice: 15 coins".to_string());
    }

    thread::sleep(Duration::from_secs(15));

    {
        let blockchain = blockchain.lock().expect("Failed to acquire lock on blockchain for printing.");
        blockchain.print_chain();
    }

    validator_thread.join().expect("Validator thread panicked.");
}
