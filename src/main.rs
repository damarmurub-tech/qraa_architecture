use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::{self, Next},
    response::Response,
    routing::post,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::signal;
use tokio::sync::RwLock;
use tracing::{error, info, warn};
use redis::AsyncCommands;

// ========================================================
// 1. TRAIT ABSTRAKSI PASCA-KUANTUM
// ========================================================
pub trait PostQuantumCrypto: Send + Sync {
    fn encrypt(&self, secret_seed: u64, entropy: u64) -> u64;
    fn adapt_dimension(&self, load_level: u32);
    fn current_dimension(&self) -> usize;
}

pub struct LatticeLweEngine {
    lattice_dimension: AtomicUsize,
    modulus_q: AtomicU64,
}

impl LatticeLweEngine {
    pub fn new(dim: usize) -> Self {
        Self {
            lattice_dimension: AtomicUsize::new(dim),
            modulus_q: AtomicU64::new(8388593),
        }
    }
}

impl PostQuantumCrypto for LatticeLweEngine {
    fn encrypt(&self, secret_seed: u64, entropy: u64) -> u64 {
        let matrix_a_approx = 1103515245_u64;
        let q = self.modulus_q.load(Ordering::Relaxed);
        (matrix_a_approx.wrapping_mul(secret_seed).wrapping_add(entropy)) % q
    }

    fn adapt_dimension(&self, load_level: u32) {
        if load_level > 1000 && self.lattice_dimension.load(Ordering::Relaxed) == 512 {
            self.lattice_dimension.store(1024, Ordering::Relaxed);
            self.modulus_q.fetch_add(50_000, Ordering::Relaxed);
            info!("[CORE-AI-REACTOR] Skala diperluas secara otomatis ke Dimensi 1024.");
        }
    }

    fn current_dimension(&self) -> usize {
        self.lattice_dimension.load(Ordering::Relaxed)
    }
}

// ========================================================
// 2. CORE AI EVOLUTION ENGINE
// ========================================================
pub struct CoreAiEvolutionEngine {
    pub registry: RwLock<HashMap<String, String>>,
}

impl CoreAiEvolutionEngine {
    pub fn new() -> Self {
        Self {
            registry: RwLock::new(HashMap::new()),
        }
    }

    pub async fn assimilate(&self, name: &str, body: &str) {
        let mut map = self.registry.write().await;
        map.insert(name.to_string(), format!("{}_ASSIMILATED_BY_CORE_AI", body));
        info!("[CORE-AI-EVOLUTION] Algoritma baru terserap: '{}'. Total: {}", name, map.len());
    }

    pub async fn total_assimilated(&self) -> usize {
        self.registry.read().await.len()
    }
}

// ========================================================
// 3. MIDDLEWARE KEAMANAN (API KEY AUTHENTICATION)
// ========================================================
async fn api_key_auth(
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let expected_key = env::var("API_KEY").unwrap_or_else(|_| "CORE_AI_SECRET_KEY_2026".to_string());

    if let Some(key) = headers.get("x-api-key") {
        if let Ok(key_str) = key.to_str() {
            if key_str == expected_key {
                return Ok(next.run(request).await);
            }
        }
    }

    warn!("[SECURITY] Akses ditolak: API Key tidak valid atau tidak ditemukan.");
    Err(StatusCode::UNAUTHORIZED)
}

// ========================================================
// 4. STATE APLIKASI GLOBAL
// ========================================================
#[derive(Clone)]
pub struct AppState {
    pub redis_conn: redis::aio::ConnectionManager,
    pub crypto_engine: Arc<dyn PostQuantumCrypto>,
    pub ai_engine: Arc<CoreAiEvolutionEngine>,
}

// ========================================================
// 5. REST API DTO & HANDLER
// ========================================================
#[derive(Deserialize)]
pub struct PacketRequestDto {
    pub request_count: u32,
    pub raw_data: String,
    pub qkd_key: u64,
    pub foreign_algo_name: Option<String>,
    pub foreign_algo_body: Option<String>,
}

#[derive(Serialize)]
pub struct PacketResponseDto {
    pub status: String,
    pub active_dimension: usize,
    pub lwe_cipher_result: u64,
    pub total_assimilated_algorithms: usize,
}

async fn handle_secure_packet(
    State(state): State<AppState>,
    Json(payload): Json<PacketRequestDto>,
) -> (StatusCode, Json<PacketResponseDto>) {
    state.crypto_engine.adapt_dimension(payload.request_count);

    if let (Some(name), Some(body)) = (payload.foreign_algo_name, payload.foreign_algo_body) {
        state.ai_engine.assimilate(&name, &body).await;
    }

    let is_qkd_valid = payload.qkd_key != 0 && payload.qkd_key % 2 == 0;
    if !is_qkd_valid {
        warn!("[CORE-AI-TEMPORAL] QKD invalid. Protocol freeze active.");
        return (
            StatusCode::BAD_REQUEST,
            Json(PacketResponseDto {
                status: "ERR_CORE_AI_PROTOCOL: TEMPORAL_FREEZE_ISOLATED".to_string(),
                active_dimension: state.crypto_engine.current_dimension(),
                lwe_cipher_result: 0,
                total_assimilated_algorithms: state.ai_engine.total_assimilated().await,
            }),
        );
    }

    let mut redis_cli = state.redis_conn.clone();
    if payload.raw_data.contains("QUANTUM_EXPLOIT") {
        warn!("[CORE-AI-EXPLOIT] Eksploit kuantum terdeteksi. Melakukan rollback.");
        let _pop_res: Result<Option<String>, _> = redis_cli.rpop("sovereign:absolute_timeline", None).await;
        return (
            StatusCode::OK,
            Json(PacketResponseDto {
                status: "CORE_AI_PROTOCOL_SUCCESS: TIMELINE_RESTORED_TO_SAFE_PAST".to_string(),
                active_dimension: state.crypto_engine.current_dimension(),
                lwe_cipher_result: 0,
                total_assimilated_algorithms: state.ai_engine.total_assimilated().await,
            }),
        );
    }

    let entropy = payload.raw_data.len() as u64 * 5_000_000_000;
    let cipher_result = state.crypto_engine.encrypt(payload.qkd_key, entropy);

    let push_res: Result<(), _> = redis_cli.rpush("sovereign:absolute_timeline", &payload.raw_data).await;
    let assimilated_count = state.ai_engine.total_assimilated().await;
    let current_dim = state.crypto_engine.current_dimension();

    match push_res {
        Ok(_) => (
            StatusCode::OK,
            Json(PacketResponseDto {
                status: "SUCCESS_DATA_COMMITTED_TO_SOVEREIGN_TIMELINE".to_string(),
                active_dimension: current_dim,
                lwe_cipher_result: cipher_result,
                total_assimilated_algorithms: assimilated_count,
            }),
        ),
        Err(e) => {
            error!("[DATABASE-ERROR] Gagal menyimpan ke Redis: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(PacketResponseDto {
                    status: "ERR_CORE_AI_PROTOCOL: REDIS_PERSISTENCE_FAIL".to_string(),
                    active_dimension: current_dim,
                    lwe_cipher_result: 0,
                    total_assimilated_algorithms: assimilated_count,
                }),
            )
        }
    }
}

// ========================================================
// 6. GRACEFUL SHUTDOWN HANDLER
// ========================================================
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Gagal memasang signal handler Ctrl+C");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Gagal memasang signal handler SIGTERM")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => info!("[SYSTEM] Sinyal Ctrl+C diterima. Memulai Graceful Shutdown..."),
        _ = terminate => info!("[SYSTEM] Sinyal SIGTERM diterima. Memulai Graceful Shutdown..."),
    }
}

// ========================================================
// 7. ENTRY POINT APLIKASI
// ========================================================
#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());

    let client = redis::Client::open(redis_url.as_str()).expect("URL Redis Tidak Valid");
    let redis_conn = redis::aio::ConnectionManager::new(client)
        .await
        .expect("Gagal menghubungkan ke Redis");

    let state = AppState {
        redis_conn,
        crypto_engine: Arc::new(LatticeLweEngine::new(512)),
        ai_engine: Arc::new(CoreAiEvolutionEngine::new()),
    };

    let app = Router::new()
        .route("/api/v1/temporal/process", post(handle_secure_packet))
        .layer(middleware::from_fn(api_key_auth))
        .with_state(state);

    let addr_str = format!("0.0.0.0:{}", port);
    let addr: SocketAddr = addr_str.parse().expect("Format socket gagal");

    info!("[CORE-AI-SYSTEM] Engine Kompatibilitas Enterprise aktif di http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}