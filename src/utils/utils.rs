use std::sync::atomic::AtomicUsize;

static COUNTER: AtomicUsize = AtomicUsize::new(1);

// generate unique key for node
pub fn generate_node_key() -> u32 {
    COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst) as u32
}

pub fn mm2px(mm: f32) -> u32 {
    (mm * 3.77952755905512).round() as u32
}