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
    is_faulty: bool, // Node'ın hatalı olup olmadığı
}

impl Node {
    fn new(name: &str, is_faulty: bool) -> Self {
        Node {
            name: name.to_string(),
            is_faulty,
        }
    }
}

struct Blockchain {
    chain: VecDeque<Block>,
    pending_transactions: Vec<String>,
    nodes: HashMap<String, Node>,
}

impl Blockchain {
    fn new() -> Self {
        let genesis_block = Block::new(0, String::from("0"), vec!["Genesis Block".to_string()], "System".to_string());
        let mut blockchain = Blockchain {
            chain: VecDeque::new(),
            pending_transactions: vec![],
            nodes: HashMap::new(),
        };
        blockchain.chain.push_back(genesis_block);
        blockchain
    }

    fn add_transaction(&mut self, transaction: String) {
        self.pending_transactions.push(transaction);
    }

    fn register_node(&mut self, node: Node) {
        self.nodes.insert(node.name.clone(), node);
    }

    fn validate_transaction(&self, transaction: &str) -> bool {
        // Tüm düğümler işlemi doğrulamalıdır
        let mut valid_votes = 0;
        let mut total_votes = 0;

        for node in self.nodes.values() {
            total_votes += 1;

            if !node.is_faulty {
                // Hatalı olmayan düğümler işlemi onaylar
                valid_votes += 1;
            }
        }

        // Çoğunluk onay verirse işlem geçerlidir
        valid_votes * 2 > total_votes
    }

    fn finalize_pending_transactions(&mut self) {
        if self.pending_transactions.is_empty() {
            return;
        }

        let mut valid_transactions = vec![];

        for transaction in &self.pending_transactions {
            if self.validate_transaction(transaction) {
                valid_transactions.push(transaction.clone());
            }
        }

        if !valid_transactions.is_empty() {
            let last_block = self.chain.back().expect("Blockchain is empty; no last block found.");
            let validator = self.select_validator();
            let new_block = Block::new(
                last_block.id + 1,
                last_block.hash.clone(),
                valid_transactions,
                validator,
            );
            self.chain.push_back(new_block);
            self.pending_transactions.clear();
        } else {
            println!("No valid transactions to finalize.");
        }
    }

    fn select_validator(&self) -> String {
        // Validator seçimi: Hatalı olmayan düğümler arasından rastgele seçilir.
        for node in self.nodes.values() {
            if !node.is_faulty {
                return node.name.clone();
            }
        }

        "System".to_string() // Eğer hiç sağlıklı düğüm yoksa, sistem blok üretir
    }

    fn print_chain(&self) {
        for block in &self.chain {
            println!("{:?}", block);
        }
    }
}

fn main() {
    let blockchain = Arc::new(Mutex::new(Blockchain::new()));

    let nodes = vec![
        Node::new("Alice", false),
        Node::new("Bob", false),
        Node::new("Charlie", true), // Hatalı bir node ekleniyor
        Node::new("David", false),
    ];

    {
        let mut blockchain = blockchain.lock().unwrap();
        for node in nodes {
            blockchain.register_node(node);
        }
    }

    // İşlem ekliyoruz
    {
        let mut blockchain = blockchain.lock().unwrap();
        blockchain.add_transaction("Alice -> Bob: 10 coins".to_string());
        blockchain.add_transaction("Bob -> Charlie: 5 coins".to_string());
    }

    // Blokları finalize ediyoruz
    {
        let mut blockchain = blockchain.lock().unwrap();
        blockchain.finalize_pending_transactions();
    }

    // Blockchain'i görüntülüyoruz
    {
        let blockchain = blockchain.lock().unwrap();
        blockchain.print_chain();
    }
}
