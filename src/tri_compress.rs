// src/tri_compress.rs
// ====================
// DRIVER NATIVO TRI - Compressão no Metal
// ====================

pub fn compress(data: &[u8; 32]) -> [u8; 64] {
    let mut compressed = [0u8; 64];
    let mut idx = 0usize;
    if data.is_empty() { return compressed; }
    let mut count: u8 = 1;
    let mut last = data[0];
    for &byte in data.iter().skip(1) {
        if byte == last && count < 255 && idx < 62 {
            count = count.saturating_add(1);
        } else {
            if idx < 62 {
                compressed[idx] = last;
                compressed[idx + 1] = count;
                idx += 2;
            }
            last = byte;
            count = 1;
        }
    }
    if idx < 62 {
        compressed[idx] = last;
        compressed[idx + 1] = count;
    }
    compressed
}

pub fn decompress(compressed: &[u8; 64]) -> [u8; 32] {
    let mut decompressed = [0u8; 32];
    let mut i = 0usize;
    let mut out_idx = 0usize;
    while i + 1 < 64 && out_idx < 32 {
        let byte = compressed[i];
        let count = (compressed[i + 1] as usize).min(32 - out_idx);
        for j in 0..count {
            decompressed[out_idx + j] = byte;
        }
        out_idx += count;
        i += 2;
    }
    decompressed
}

pub fn compress_str(input: &str) -> [u8; 64] {
    let mut data = [0u8; 32];
    let bytes = input.as_bytes();
    let len = bytes.len().min(32);
    data[..len].copy_from_slice(&bytes[..len]);
    compress(&data)
}

// Fix: Stats único (sem duplicate, _original pra unused)
pub fn stats(_original: &[u8; 32], compressed: &[u8; 64]) -> (u32, u32, u8) {
    let orig_len = 32u32;
    let comp_len = compressed.iter().position(|&x| x == 0).unwrap_or(64) as u32;
    let ratio = if comp_len > 0 { (orig_len * 100 / comp_len) as u8 } else { 100 };
    (orig_len, comp_len, ratio)
}
