use chrono::{Utc};
use sha2::{Sha256, Digest};
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
    hash: String,
    validator: String,
}

impl Block {
    fn new(id: u64, previous_hash: String, transactions: Vec<String>, validator: String) -> Self {
        let timestamp = Utc::now().to_rfc3339();
        let mut block = Block {
            id,
            timestamp,
            previous_hash,
            transactions,
            hash: String::new(),
            validator,
        };
        block.finalize_block();
        block
    }

    fn calculate_hash(&self) -> String {
        let data = format!(
            "{}{}{}{}{}",
            self.id, self.timestamp, self.previous_hash, self.validator, self.transactions.join("")
        );
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }

    fn finalize_block(&mut self) {
        self.hash = self.calculate_hash();
    }
}

#[derive(Debug, Clone)]
struct Node {
    name: String,
    stake: u64,
    votes: u64,
}

impl Node {
    fn new(name: &str, stake: u64) -> Self {
        Node {
            name: name.to_string(),
            stake,
            votes: 0,
        }
    }
}

struct Blockchain {
    chain: VecDeque<Block>,
    pending_transactions: Vec<String>,
    nodes: HashMap<String, Node>,
    delegates: Vec<String>,
}

impl Blockchain {
    fn new() -> Self {
        let genesis_block = Block::new(0, String::from("0"), vec!["Genesis Block".to_string()], "System".to_string());
        let mut blockchain = Blockchain {
            chain: VecDeque::new(),
            pending_transactions: vec![],
            nodes: HashMap::new(),
            delegates: vec![],
        };
        blockchain.chain.push_back(genesis_block);
        blockchain
    }

    fn register_node(&mut self, node: Node) {
        self.nodes.insert(node.name.clone(), node);
    }

    fn vote_for_delegate(&mut self, voter: &str, delegate: &str) {
        let voter_stake = if let Some(voter_node) = self.nodes.get(voter) {
            voter_node.stake
        } else {
            println!("Voter {} not found.", voter);
            return;
        };

        if let Some(delegate_node) = self.nodes.get_mut(delegate) {
            delegate_node.votes += voter_stake; // Voter'ın stake'i kadar oy eklenir
            println!("{} voted for {}", voter, delegate);
        } else {
            println!("Delegate {} not found.", delegate);
        }
    }

    fn elect_delegates(&mut self, num_delegates: usize) {
        let mut candidates: Vec<_> = self.nodes.values().collect();
        candidates.sort_by(|a, b| b.votes.cmp(&a.votes)); // En çok oy alanları sıralar
        self.delegates = candidates
            .into_iter()
            .take(num_delegates)
            .map(|node| node.name.clone())
            .collect();
        println!("Elected delegates: {:?}", self.delegates);
    }

    fn add_transaction(&mut self, transaction: String) {
        self.pending_transactions.push(transaction);
    }

    fn finalize_pending_transactions(&mut self) {
        if self.delegates.is_empty() {
            println!("No delegates selected. Cannot finalize transactions.");
            return;
        }

        let validator = self.delegates[0].clone(); // İlk delege blokları üretir
        let last_block = self.chain.back().expect("Blockchain is empty; no last block found.");
        let new_block = Block::new(
            last_block.id + 1,
            last_block.hash.clone(),
            self.pending_transactions.clone(),
            validator,
        );
        self.chain.push_back(new_block);
        self.pending_transactions.clear();
    }

    fn print_chain(&self) {
        for block in &self.chain {
            println!("{:?}", block);
        }
    }
}

fn main() {
    let blockchain = Arc::new(Mutex::new(Blockchain::new()));

    // Node'lar kaydediliyor
    let nodes = vec![
        Node::new("Alice", 50),
        Node::new("Bob", 30),
        Node::new("Charlie", 20),
        Node::new("David", 10),
    ];

    {
        let mut blockchain = blockchain.lock().unwrap();
        for node in nodes {
            blockchain.register_node(node);
        }
    }

    // Oy verme işlemi
    {
        let mut blockchain = blockchain.lock().unwrap();
        blockchain.vote_for_delegate("Alice", "Bob");
        blockchain.vote_for_delegate("Charlie", "Bob");
        blockchain.vote_for_delegate("David", "Alice");
        blockchain.elect_delegates(2); // En çok oy alan 2 kişi seçiliyor
    }

    // İşlem ekleme ve blok oluşturma
    {
        let mut blockchain = blockchain.lock().unwrap();
        blockchain.add_transaction("Alice -> Bob: 10 coins".to_string());
        blockchain.add_transaction("Bob -> Charlie: 5 coins".to_string());
        blockchain.add_transaction("Charlie -> Alice: 15 coins".to_string());
    }

    thread::sleep(Duration::from_secs(2)); // Bir süre bekleme

    {
        let mut blockchain = blockchain.lock().unwrap();
        blockchain.finalize_pending_transactions(); // Blok üretiliyor
    }

    // Blockchain görüntüleme
    {
        let blockchain = blockchain.lock().unwrap();
        blockchain.print_chain();
    }
}
