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
struct Authority {
    name: String,
    approved: bool,
}

impl Authority {
    fn new(name: &str) -> Self {
        Authority {
            name: name.to_string(),
            approved: true,
        }
    }
}

struct Blockchain {
    chain: VecDeque<Block>,
    pending_transactions: Vec<String>,
    authorities: HashMap<String, Authority>,
}

impl Blockchain {
    fn new() -> Self {
        let genesis_block = Block::new(0, String::from("0"), vec!["Genesis Block".to_string()], "System".to_string());
        let mut blockchain = Blockchain {
            chain: VecDeque::new(),
            pending_transactions: vec![],
            authorities: HashMap::new(),
        };
        blockchain.chain.push_back(genesis_block);
        blockchain
    }

    fn register_authority(&mut self, authority: Authority) {
        self.authorities.insert(authority.name.clone(), authority);
    }

    fn add_transaction(&mut self, transaction: String) {
        self.pending_transactions.push(transaction);
    }

    fn select_validator(&self) -> Option<String> {
        // İlk onaylanmış otoriteyi seçer.
        for authority in self.authorities.values() {
            if authority.approved {
                return Some(authority.name.clone());
            }
        }
        None
    }

    fn finalize_pending_transactions(&mut self) {
        let validator = match self.select_validator() {
            Some(v) => v,
            None => {
                println!("No approved authorities available to validate transactions.");
                return;
            }
        };

        let last_block = self.chain.back().expect("Blockchain is empty; no last block found.");
        let new_block = Block::new(
            last_block.id + 1,
            last_block.hash.clone(),
            self.pending_transactions.clone(),
            validator,
        );

        // Çözüm 1: `clone` kullanılarak hatadan kaçınılır
        self.chain.push_back(new_block.clone()); // Blok eklenmeden önce kopyalanır
        self.pending_transactions.clear();
        println!("Block finalized by: {}", new_block.validator); // Blok bilgisi yazdırılır

        // Çözüm 2 (yorum satırı olarak): Validator bilgisini saklayarak daha verimli bir çözüm
        // let validator = new_block.validator.clone(); // Validator bilgisi saklanıyor
        // self.chain.push_back(new_block);            // new_block taşınıyor
        // println!("Block finalized by: {}", validator); // Validator bilgisi kullanılıyor
    }

    fn print_chain(&self) {
        for block in &self.chain {
            println!("{:?}", block);
        }
    }
}

fn main() {
    let blockchain = Arc::new(Mutex::new(Blockchain::new()));
    
    let authorities = vec![
        Authority::new("Alice"),
        Authority::new("Bob"),
    ];

    {
        let mut blockchain = blockchain.lock().unwrap();
        for authority in authorities {
            blockchain.register_authority(authority);
        }
    }

    {
        let mut blockchain = blockchain.lock().unwrap();
        blockchain.add_transaction("Alice -> Bob: 10 coins".to_string());
        blockchain.add_transaction("Bob -> Charlie: 5 coins".to_string());
    }

    {
        let mut blockchain = blockchain.lock().unwrap();
        blockchain.finalize_pending_transactions();
    }

    {
        let blockchain = blockchain.lock().unwrap();
        blockchain.print_chain();
    }
}
