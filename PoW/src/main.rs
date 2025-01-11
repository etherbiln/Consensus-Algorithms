use chrono::{Utc, TimeZone};
use sha2::{Sha256, Digest}; // sha2 crate'ini içe aktarıyoruz
use std::collections::VecDeque;
use std::fmt;

#[derive(Debug, Clone)]
pub struct Block {
    pub index: u32,
    pub previous_hash: String,
    pub timestamp: String,
    pub transactions: Vec<String>,  // Basitlik açısından işlemler bir liste olarak tutuluyor
    pub nonce: u32,
    pub hash: String,
}

impl Block {
    fn new(index: u32, previous_hash: String, transactions: Vec<String>) -> Self {
        let timestamp = Utc::now().to_rfc3339();
        Block {
            index,
            previous_hash,
            timestamp,
            transactions,
            nonce: 0,
            hash: String::new(), // Hash başlangıçta boş
        }
    }

    // Blok hash'ini hesaplayacak fonksiyon
    fn calculate_hash(&self) -> String {
        let data = format!(
            "{}{}{}{}{}",
            self.index, self.previous_hash, self.timestamp, self.nonce, self.transactions.join("")
        );
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }

    // Proof of Work algoritmasını uygulayarak uygun nonce'u bulacak fonksiyon
    pub fn mine_block(&mut self, difficulty: usize) {
        self.hash = self.calculate_hash(); // İlk olarak hash'i hesapla

        let target = "0".repeat(difficulty);  // Belirli bir zorluk seviyesini hedefliyoruz
        while &self.hash[..difficulty] != target {
            self.nonce += 1;
            self.hash = self.calculate_hash(); // Yeni nonce değeri ile hash hesapla
        }
        println!("Block mined: {} with nonce: {}", self.hash, self.nonce);
    }
}

// Blockchain yapısı
pub struct Blockchain {
    pub chain: VecDeque<Block>,
    pub difficulty: usize,
}

impl Blockchain {
    pub fn new(difficulty: usize) -> Self {
        let mut blockchain = Blockchain {
            chain: VecDeque::new(),
            difficulty,
        };
        
        // Genesis bloğu (ilk blok) ekliyoruz
        let genesis_block = Block::new(0, String::from("0"), vec!["Genesis Block".into()]);
        blockchain.chain.push_back(genesis_block);
        
        blockchain
    }

    // Yeni blok eklemek
    pub fn add_block(&mut self, transactions: Vec<String>) {
        let last_block = self.chain.back().unwrap();
        let mut new_block = Block::new(last_block.index + 1, last_block.hash.clone(), transactions);
        new_block.mine_block(self.difficulty); // Blok madenleme işlemi
        self.chain.push_back(new_block);
    }

    // Zinciri yazdırmak
    pub fn print_chain(&self) {
        for block in &self.chain {
            println!("{:?}", block);
        }
    }
}

fn main() {
    let mut blockchain = Blockchain::new(4);
    println!("Blockchain created with difficulty : 4");

    blockchain.add_block(vec!["Transaction 1".into()]);
    blockchain.add_block(vec!["Transaction 2".into()]);
    blockchain.add_block(vec!["Transaction 3".into(),"Transaction 4".into()]);


    blockchain.print_chain();
}

