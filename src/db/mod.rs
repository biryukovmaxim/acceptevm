use crate::common::DatabaseError;
use crate::{audit::log_sync, types::Serializable};
use sled::Tree;

/// Retrieve a value by key from a tree.
async fn get_from_tree(db: &Tree, key: &str) -> Result<Vec<u8>, DatabaseError> {
    match db.get(key) {
        Ok(result) => match result {
            Some(value) => Ok(value.to_vec()),
            None => Err(DatabaseError::NotFound),
        },
        Err(_error) => Err(DatabaseError::Get),
    }
}
/// Retrieve all key,value pairs from a specified tree
async fn get_all_from_tree(db: &Tree) -> Result<Vec<(Vec<u8>, Vec<u8>)>, DatabaseError> {
    let mut all = Vec::new();
    for el in db.iter() {
        match el {
            Ok(value) => {
                let el_bin_key = value.0.to_vec();
                let el_bin_value = value.1.to_vec();
                all.push((el_bin_key, el_bin_value));
            }
            Err(error) => {
                log_sync(&format!("Db Interaction Error: {}", error));
                return Err(DatabaseError::Get);
            }
        }
    }
    Ok(all)
}

/// Retrieve the last added item to the tree
async fn get_last_from_tree(db: &Tree) -> Result<(Vec<u8>, Vec<u8>), DatabaseError> {
    match db.last() {
        Ok(value) => match value {
            Some(tuple) => {
                let el_bin_key = tuple.0.to_vec();
                let el_bin_value = tuple.1.to_vec();
                Ok((el_bin_key, el_bin_value))
            }
            None => Err(DatabaseError::NotFound),
        },
        Err(error) => {
            log_sync(&format!("Db Interaction Error: {}", error));
            Err(DatabaseError::Get)
        }
    }
}

/// Wrapper for retrieving the last added item to the tree
pub async fn get_last<T: Serializable>(tree: &sled::Tree) -> Result<(String, T), DatabaseError> {
    let binary_data = get_last_from_tree(tree).await?;
    // Convert binary key to String
    let key = String::from_utf8(binary_data.0).map_err(|error| {
        log_sync(&format!("Db Interaction Error: {}", error));
        DatabaseError::Deserialize
    })?;

    // Deserialize binary value to T
    let value = T::from_bin(binary_data.1).map_err(|error| {
        log_sync(&format!("Db Interaction Error: {}", error));
        DatabaseError::Deserialize
    })?;
    Ok((key, value))
}

/// Wrapper for retrieving all key value pairs from a tree
pub async fn get_all<T: Serializable>(
    tree: &sled::Tree,
) -> Result<Vec<(String, T)>, DatabaseError> {
    let binary_data = get_all_from_tree(tree).await?;
    let mut all = Vec::new();
    for (binary_key, binary_value) in binary_data {
        // Convert binary key to String
        let key = String::from_utf8(binary_key.to_vec()).map_err(|error| {
            log_sync(&format!("Db Interaction Error: {}", error));
            DatabaseError::Deserialize
        })?;

        // Deserialize binary value to T
        let value = T::from_bin(binary_value).map_err(|error| {
            log_sync(&format!("Db Interaction Error: {}", error));
            DatabaseError::Deserialize
        })?;

        all.push((key, value));
    }
    Ok(all)
}

/// Wrapper for retrieving a value from a tree
pub async fn get<T: Serializable>(tree: &Tree, key: &str) -> Result<T, DatabaseError> {
    let binary_data = get_from_tree(tree, key).await?;
    T::from_bin(binary_data).map_err(|error| {
        log_sync(&format!("Db Interaction Error: {}", error));
        DatabaseError::Deserialize
    })
}

/// Sets a value to a tree
async fn set_to_tree(db: &Tree, key: &str, bin: Vec<u8>) -> Result<(), DatabaseError> {
    match db.insert(key, bin) {
        Ok(_) => Ok(()),
        Err(error) => {
            log_sync(&format!("Db Interaction Error: {}", error));
            Err(DatabaseError::Set)
        }
    }
}

/// Wrapper for setting a value to a tree
pub async fn set<T: Serializable>(tree: &Tree, key: &str, data: T) -> Result<(), DatabaseError> {
    let binary_data = T::to_bin(&data).map_err(|error| {
        log_sync(&format!("Db Interaction Error: {}", error));
        DatabaseError::Serialize
    })?;
    set_to_tree(tree, key, binary_data)
        .await
        .map_err(|_| DatabaseError::Communicate)?;
    Ok(())
}

/// Used to delete from a tree
pub async fn delete(tree: &Tree, key: &str) -> Result<(), DatabaseError> {
    match tree.remove(key) {
        Ok(result) => match result {
            Some(_deleted_value) => Ok(()),
            None => Err(DatabaseError::NotFound),
        },
        Err(error) => {
            log_sync(&format!("Db Interaction Error: {}", error));
            Err(DatabaseError::NoDelete)
        }
    }
}
