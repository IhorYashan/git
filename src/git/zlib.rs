extern crate hex;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use sha1::{Digest, Sha1};
use std::io::Read;
use std::io::Write;

// error Handling

pub fn decode_data(compressed_data: &[u8]) -> (Vec<u8>, usize) {
    let mut decoder = ZlibDecoder::new(compressed_data);
    let mut buff_vec = Vec::new();
    decoder.read_to_end(&mut buff_vec).unwrap();
    let bytes_read = decoder.total_in();
    (buff_vec, bytes_read as usize)
}

pub fn encode_data(data_to_compress: String) -> (String, Vec<u8>) {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(data_to_compress.as_bytes()).unwrap();
    let compressed_data = encoder.finish().unwrap();
    let mut hasher = Sha1::new();
    hasher.update(data_to_compress);
    let hash = hasher.finalize();
    let hash_file = hex::encode(&hash);
    (hash_file, compressed_data)
}
