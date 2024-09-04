use core::panic;
use std::{env, io::{stdin, stdout, Write}};
use chrono::prelude::*;
use log::warn;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const MAX_TRANSACTIONS_IN_A_BLOCK: usize = 5; 
const SCROLL_MINIMUM_SIGNATURES: usize = 3;

pub struct App {
    pub blocks: Vec<Block>,
    pub new_block: Option<Block>,
    pub scroll: Option<BlockScroll>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Transaction{
    from:String,
    to: String,
    amount: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    pub id: u64,
    pub hash: String,
    pub previous_hash: String,
    pub timestamp: i64,
    pub data: Vec<Transaction>,    
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Signature{
    pub signer: String,
    pub signature: String
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct  BlockScroll{
    pub block:Block,
    pub signatures: Vec<Signature>
}

impl App{
    fn new() -> Self{
        Self {blocks: vec![] , new_block: None, scroll: None }
    }

    fn genesis(&mut self){
        let genesis_block = Block {
            id: 0,
            timestamp: Utc::now().timestamp(),
            previous_hash: String::from("genesis"),
            data: vec![],            
            hash: "0000f816a87f806bb0073dcf026a64fb40c946b5abee2573702828694d5b4c43".to_string(),
        };
        self.blocks.push(genesis_block);
    }    

    fn add_transaction(&mut self,from: String,to: String,amount: u64){
        let transaction = Transaction{
            amount: amount,
            from: from,
            to: to
        };
        
        match &mut self.new_block {
            None =>{                
                match  self.blocks.last() {
                    None=>panic!("There is no previous block!"),
                    Some(previous_block) => {
                        let previous_block_hash = hex::encode(calculate_hash(previous_block));
                        self.new_block = Some(Block::new(1, previous_block_hash, vec![]));
                    }
                }
            },
            Some(block)=>{
                block.data.push(transaction);
                if block.data.len() == MAX_TRANSACTIONS_IN_A_BLOCK {
                    self.scroll = Some( BlockScroll{
                        block: self.new_block.as_ref().unwrap().clone(),
                        signatures: vec![]
                    });
                    self.new_block = None;                    
                }                    
            } 
        }        
    }
    

    fn sign_scroll(&mut self,signer:String,signature: String){
        let sig = Signature{
            signer,
            signature
        };
        match &mut self.scroll{
            None=> panic!("No scroll to sign"),
            Some(scroll) =>{
                if scroll.signatures.len() >= SCROLL_MINIMUM_SIGNATURES {                    
                    scroll.signatures.push(sig);     
                    if is_block_valid(self.blocks.last().unwrap(),&scroll.block){
                        self.blocks.push(scroll.block.clone());
                        self.scroll = None;                
                        println!("Scroll now has enough signature now.");
                    }else{
                        panic!("Trying to add invalid block to the blockchain");
                    }                        
                }else{
                    println!("Scroll remaining sigature: {}.",SCROLL_MINIMUM_SIGNATURES - scroll.signatures.len());
                }
            }
        }
    }
}

impl Block{
    pub fn new(id: u64, previous_hash: String, data: Vec<Transaction>) -> Self {
        let now = Utc::now();        
        Self {
            id,            
            hash: String::from(""),
            timestamp: now.timestamp(),            
            previous_hash,
            data,
        }
    }
}

fn calculate_hash(block:&Block) -> Vec<u8> {
    let data = serde_json::to_string_pretty(block);
    let mut hasher = Sha256::new();
    hasher.update(data.unwrap().as_bytes());
    hasher.finalize().as_slice().to_owned()
}

fn is_block_valid( block: &Block, previous_block: &Block) -> bool {
    if block.previous_hash != previous_block.hash {
        warn!("block with id: {} has wrong previous hash", block.id);
        return false;
    }else if block.id != previous_block.id + 1 {
        warn!(
            "block with id: {} is not the next block after the latest: {}",
            block.id, previous_block.id
        );
        return false;
    } else if hex::encode(calculate_hash(&block)) != block.hash
    {
        warn!("block with id: {} has invalid hash", block.id);
        return false;
    }
    true
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChainResponse {
    pub blocks: Vec<Block>,
    pub receiver: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LocalChainRequest {
    pub from_peer_id: String,
}

pub enum EventType {
    LocalChainResponse(ChainResponse),
    Input(String),
    Init,
}

fn main() {
    let mut app = App::new();
    env::set_var("RUST_LOG", "info");
    pretty_env_logger::init();
    println!("Enter \"help\" for commands");    
    let mut cmd;

    app.genesis();      

    loop{
        print!("~:");
        cmd = String::from("");
        stdout().flush().unwrap();
        stdin().read_line(&mut cmd).ok().expect("Failed to read the command.");        

        match cmd.trim() {
            "help"=>{
                println!("help: shows list of commands");
                println!("ls: shows list of blocks");
                println!("transfer: transfer an amount from an account to another");
                println!("sign: sign a block");
                println!("quit: quit the program");                
            },
            "ls"=>{
                let pretty_json = serde_json::to_string_pretty(&app.blocks).expect("can't jsonify blocks");
                println!("{}",pretty_json);
            },
            "transfer"=>{
                let mut from = String::from("");
                
                let mut to = String::from("");                
                
                let amount:u64;
                let mut buff = String::from("");

                let mut signaure = String::from("");

                print!("From:");                
                stdout().flush().unwrap();
                stdin().read_line(&mut from).ok().expect("Failed to read the From.");        

                print!("To:");                
                stdout().flush().unwrap();
                stdin().read_line(&mut to).ok().expect("Failed to read the To.");        

                print!("Amount:");                
                stdout().flush().unwrap();
                stdin().read_line(&mut buff).ok().expect("Failed to read the Amount.");        
                amount = buff.parse().unwrap();

                print!("Signature:");                
                stdout().flush().unwrap();
                stdin().read_line(&mut signaure).ok().expect("Failed to read the Signature.");        
                //TODO: verify signature to make sure the transactions signed by sender
                app.add_transaction(from,to,amount);
            },
            "sign"=>{
                let mut signer = String::from("");
                let mut signature = String::from("");

                print!("Signer (public key):");                
                stdout().flush().unwrap();
                stdin().read_line(&mut signer).ok().expect("Failed to read the Signer.");        

                print!("Signature:");                
                stdout().flush().unwrap();
                stdin().read_line(&mut signature).ok().expect("Failed to read the Signature.");        

                app.sign_scroll(signer,signature);
            },
            "quit"=>{
                println!("Goodbye.");
                break;
            },
            _=>println!("Unkown command \"{}\".",cmd.trim())
        }              
    }
}
