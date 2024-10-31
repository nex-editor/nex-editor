use std::sync::atomic::AtomicUsize;

static COUNTER: AtomicUsize = AtomicUsize::new(1);

// 调用一次加一次
pub fn generate_node_key() -> u32 {
    COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst) as u32
}
