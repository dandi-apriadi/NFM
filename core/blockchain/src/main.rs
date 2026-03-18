mod block;
use block::Block;

struct Blockchain {
    chain: Vec<Block>,
}

impl Blockchain {
    fn new() -> Self {
        let genesis_block = Block::new(0, "NFM GENESIS BLOCK - The Birth of Sovereign AI".to_string(), "0".to_string());
        Blockchain {
            chain: vec![genesis_block],
        }
    }

    fn add_block(&mut self, mut new_block: Block) {
        new_block.previous_hash = self.get_latest_block().hash.clone();
        new_block.hash = new_block.calculate_hash();
        self.chain.push(new_block);
    }

    fn get_latest_block(&self) -> &Block {
        self.chain.last().expect("Chain should not be empty")
    }

    fn is_chain_valid(&self) -> bool {
        for i in 1..self.chain.len() {
            let current = &self.chain[i];
            let previous = &self.chain[i - 1];

            if current.hash != current.calculate_hash() {
                return false;
            }

            if current.previous_hash != previous.hash {
                return false;
            }
        }
        true
    }
}

fn main() {
    println!("==========================================");
    println!("NFM ALPHA CORE [RUST] - Starting Blockchain");
    println!("==========================================");

    let mut nfm_chain = Blockchain::new();

    println!("Mining block 1...");
    nfm_chain.add_block(Block::new(1, "First Transaction: 100 Gold".to_string(), "".to_string()));

    println!("Mining block 2...");
    nfm_chain.add_block(Block::new(2, "Second Transaction: 50 Gold".to_string(), "".to_string()));

    for block in &nfm_chain.chain {
        println!("{:?}", block);
    }

    // Note: self.chain.length() was a typo, should be len()
}
