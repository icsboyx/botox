fn test() {
    // The object that we will serialize.
    let target: String = "hello world".to_string();
    let encoded: Vec<u8> = bincode::serialize(&target).unwrap();
    let decoded: String = bincode::deserialize(&encoded[..]).unwrap();
    assert_eq!(target, decoded);
}
