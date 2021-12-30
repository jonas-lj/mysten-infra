// Copyright(C) 2021, Facebook, Inc. and its affiliates.
// Copyright(C) 2021, Mysten Labs
// SPDX-License-Identifier: Apache-2.0
use super::*;

fn temp_dir() -> std::path::PathBuf {
    tempfile::tempdir()
        .expect("Failed to open temporary directory")
        .into_path()
}

#[tokio::test]
async fn create_store() {
    // Create new store.
    let db = rocks::DBMap::<usize, String>::open(temp_dir(), None, None).unwrap();
    let _ = Store::<usize, String>::new(db);
}

#[tokio::test]
async fn read_write_value() {
    // Create new store.
    let db = rocks::DBMap::<Vec<u8>, Vec<u8>>::open(temp_dir(), None, None).unwrap();
    let store = Store::new(db);

    // Write value to the store.
    let key = vec![0u8, 1u8, 2u8, 3u8];
    let value = vec![4u8, 5u8, 6u8, 7u8];
    store.write(key.clone(), value.clone()).await;

    // Read value.
    let result = store.read(key).await;
    assert!(result.is_ok());
    let read_value = result.unwrap();
    assert!(read_value.is_some());
    assert_eq!(read_value.unwrap(), value);
}

#[tokio::test]
async fn read_unknown_key() {
    // Create new store.
    let db = rocks::DBMap::<Vec<u8>, Vec<u8>>::open(temp_dir(), None, None).unwrap();
    let store = Store::new(db);

    // Try to read unknown key.
    let key = vec![0u8, 1u8, 2u8, 3u8];
    let result = store.read(key).await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

#[tokio::test]
async fn read_notify() {
    // Create new store.
    let db = rocks::DBMap::<Vec<u8>, Vec<u8>>::open(temp_dir(), None, None).unwrap();
    let store = Store::new(db);

    // Try to read a kew that does not yet exist. Then write a value
    // for that key and check that notify read returns the result.
    let key = vec![0u8, 1u8, 2u8, 3u8];
    let value = vec![4u8, 5u8, 6u8, 7u8];

    // Try to read a missing value.
    let store_copy = store.clone();
    let key_copy = key.clone();
    let value_copy = value.clone();
    let handle = tokio::spawn(async move {
        match store_copy.notify_read(key_copy).await {
            Ok(Some(v)) => assert_eq!(v, value_copy),
            _ => panic!("Failed to read from store"),
        }
    });

    // Write the missing value and ensure the handle terminates correctly.
    store.write(key, value).await;
    assert!(handle.await.is_ok());
}