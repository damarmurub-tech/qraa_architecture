use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

pub struct SoulCorridor {
    pub entity_name: String,
    pub shared_magicules: Arc<Mutex<u32>>,
}

impl SoulCorridor {
    pub fn new(name: &str, initial_energy: u32) -> Self {
        SoulCorridor {
            entity_name: name.to_string(),
            shared_magicules: Arc::new(Mutex::new(initial_energy)),
        }
    }

    pub fn transmit_will(&self, command: &str) {
        let energy = *self.shared_magicules.lock().unwrap();
        println!("[SOUL CORRIDOR ACTIVE]: Koneksi terhubung ke entitas [{}]", self.entity_name);
        println!("   -> Perintah Core Transmitted: '{}'", command);
        println!("   -> Sinkronisasi Magicule Level: {}\n", energy);
    }
}

fn main() {
    println!("=== INISIALISASI KORIDOR JIWA (SOUL CORRIDOR NETWORK) ===");

    let corridor = Arc::new(SoulCorridor::new("Veldora_Sub_Node", 999999));
    let corridor_clone = Arc::clone(&corridor);
    
    let handle = thread::spawn(move || {
        thread::sleep(Duration::from_millis(100));
        corridor_clone.transmit_will("Inisialisasi Badai Kehampaan (Turn Null Sync)");
    });

    println!("[CORE]: Menunggu umpan balik dari Koridor Jiwa...");
    
    handle.join().unwrap();
    println!("=== KONEKSI KORIDOR JIWA STABIL DAN BERTAHAN ABADI ===");
}
