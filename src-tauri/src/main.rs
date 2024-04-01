// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use aws_config::meta::region::RegionProviderChain;
use aws_config::profile::ProfileFileCredentialsProvider;
use aws_sdk_sts::Client as StsClient;

use ini::configparser::ini::Ini;
use serde::{Deserialize, Serialize};

use std::{env, fs, path::Path};

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn list_aws_profiles() -> Result<Vec<String>, String> {
    let home_dir = env::var("HOME").map_err(|_| "Unable to find HOME directory".to_string())?;
    let credentials_path = Path::new(&home_dir).join(".aws/credentials");

    let contents = fs::read_to_string(credentials_path)
        .map_err(|_| "Unable to read AWS credentials file".to_string())?;

    let profiles = contents
        .lines()
        .filter_map(|line| {
            if line.starts_with("[") && line.ends_with("]") {
                Some(line[1..line.len() - 1].to_string())
            } else {
                None
            }
        })
        .collect::<Vec<String>>();

    Ok(profiles)
}

#[derive(Serialize)]
struct ProfileDetails {
    access_key_id: String,
    secret_access_key: String,
    session_token: Option<String>,
}

#[tauri::command]
fn get_aws_profile_details(profile: &str) -> Result<ProfileDetails, String> {
    let home_dir = match std::env::var("HOME") {
        Ok(dir) => dir,
        Err(_) => return Err("Unable to find HOME directory".to_string()),
    };

    let credentials_path = Path::new(&home_dir).join(".aws/credentials");
    let mut config = Ini::new();
    if credentials_path.exists() {
        match config.load(credentials_path.to_str().unwrap()) {
            Ok(_) => (),
            Err(e) => return Err(format!("Failed to load AWS credentials file: {}", e)),
        }
    }

    let access_key_id = config.get(profile, "aws_access_key_id")
        .ok_or_else(|| format!("Access Key ID not found for profile: {}", profile))?;
    let secret_access_key = config.get(profile, "aws_secret_access_key")
        .ok_or_else(|| format!("Secret Access Key not found for profile: {}", profile))?;
    let session_token = config.get(profile, "aws_session_token"); // Session token is optional

    Ok(ProfileDetails {
        access_key_id: access_key_id.to_owned(),
        secret_access_key: secret_access_key.to_owned(),
        session_token: session_token.map(|s| s.to_owned()),
    })
}

#[derive(Clone)]
#[derive(Deserialize)]
struct AwsProfile {
    #[serde(rename = "profileName")]
    profile_name: String,
    #[serde(rename = "accessKeyId")]
    access_key_id: String,
    #[serde(rename = "secretAccessKey")]
    secret_access_key: String,
    #[serde(rename = "sessionToken")]
    session_token: Option<String>,
}

#[tauri::command]
fn add_or_edit_aws_profile(profile: AwsProfile) -> Result<(), String> {
    println!("Adding or editing profile: {}, Access Key ID: {}, Secret Access Key provided: {}", profile.profile_name, profile.access_key_id, !profile.secret_access_key.is_empty());
    let home_dir = match std::env::var("HOME") {
        Ok(dir) => dir,
        Err(_) => return Err("Unable to find HOME directory".to_string()),
    };

    let aws_dir = std::path::Path::new(&home_dir).join(".aws");
    let credentials_path = aws_dir.join("credentials");

    // Ensure .aws directory and credentials file exist
    if !aws_dir.exists() {
        std::fs::create_dir_all(&aws_dir).map_err(|e| format!("Failed to create .aws directory: {}", e))?;
    }
    if !credentials_path.exists() {
        std::fs::File::create(&credentials_path).map_err(|e| format!("Failed to create AWS credentials file: {}", e))?;
    }

    let mut config = Ini::new();
    // Load existing configuration if available
    if let Err(e) = config.load(credentials_path.to_str().unwrap()) {
        return Err(format!("Failed to load existing AWS credentials file: {}", e));
    }

    // Remove the existing section (if any) to ensure fresh values
    config.remove_section(&profile.profile_name);

    // Set new values for the profile
    config.set(&profile.profile_name, "aws_access_key_id", Some(profile.access_key_id.to_string()));
    config.set(&profile.profile_name, "aws_secret_access_key", Some(profile.secret_access_key.to_string()));
    if let Some(token) = profile.session_token {
        config.set(&profile.profile_name, "aws_session_token", Some(token.to_string()));
    }

    // Write the updated configuration back to the file
    match config.write(&credentials_path.to_str().unwrap()) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Failed to write to AWS credentials file: {}", e)),
    }
}

#[tauri::command]
fn delete_aws_profile(profile: &str) -> Result<(), String> {
    let home_dir = match std::env::var("HOME") {
        Ok(dir) => dir,
        Err(_) => return Err("Unable to find HOME directory".to_string()),
    };

    let credentials_path = Path::new(&home_dir).join(".aws/credentials");
    let mut config = Ini::new();
    if credentials_path.exists() {
        match config.load(credentials_path.to_str().unwrap()) {
            Ok(_) => (),
            Err(e) => return Err(format!("Failed to load AWS credentials file: {}", e)),
        }
    }

    config.remove_section(profile);

    match config.write(&credentials_path.to_str().unwrap()) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Failed to update AWS credentials file: {}", e)),
    }
}

async fn get_caller_identity(
    client: &StsClient,
) -> Result<(String, String, String), aws_sdk_sts::Error> {
    let response = client.get_caller_identity().send().await?;

    let arn = response.arn().unwrap_or_default().to_string();
    let user_id = response.user_id().unwrap_or_default().to_string();
    let account = response.account().unwrap_or_default().to_string();

    Ok((arn, user_id, account))
}

#[tauri::command]
async fn check_aws_identity(profile: &str) -> Result<String, String> {
    let credentials_provider = ProfileFileCredentialsProvider::builder()
        .profile_name(profile)
        .build();

    let region_provider = RegionProviderChain::default_provider().or_else("us-east-1");

    let shared_config = aws_config::from_env()
        .credentials_provider(credentials_provider)
        .region(region_provider)
        .load()
        .await;

    let client = StsClient::new(&shared_config);

    match get_caller_identity(&client).await {
        Ok((arn, user_id, account)) => Ok(format!(
            "ARN: {}, User ID: {}, Account: {}",
            arn, user_id, account
        )),
        Err(error) => Err(format!("Failed to get AWS caller identity: {}", error)),
    }
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            list_aws_profiles,
            get_aws_profile_details,
            add_or_edit_aws_profile,
            delete_aws_profile,
            check_aws_identity
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use serial_test::serial;

    #[test]
    #[serial]
    fn test_list_aws_profiles() {
        let temp_dir = tempdir().expect("Failed to create a temp directory");
        std::env::set_var("HOME", temp_dir.path());

        // Create a credentials file with two profiles
        let default_profile = AwsProfile {
            profile_name: "tester".to_string(),
            access_key_id: "test_access_key_id".to_string(),
            secret_access_key: "test_secret_access_key".to_string(),
            session_token: None,
        };
        assert_eq!(add_or_edit_aws_profile(default_profile.clone()).unwrap(), ());
        let test_profile = AwsProfile {
            profile_name: "test-profile".to_string(),
            access_key_id: "test_access_key_id".to_string(),
            secret_access_key: "test_secret_access_key".to_string(),
            session_token: None,
        };
        assert_eq!(add_or_edit_aws_profile(test_profile.clone()).unwrap(), ());

        // Execute the function under test
        let profiles = list_aws_profiles().unwrap();

        // Verify the results (dont care about ordering of the profiles in the vector
        assert_eq!(profiles.len(), 2);
        assert!(profiles.contains(&"tester".to_string()));
        assert!(profiles.contains(&"test-profile".to_string()));
    }

    #[test]
    #[serial]
    fn test_get_aws_profile_details() {
        let temp_dir = tempdir().expect("Failed to create a temp directory");
        std::env::set_var("HOME", temp_dir.path());
    
        let test_profile = AwsProfile {
            profile_name: "test-profile".to_string(),
            access_key_id: "12345".to_string(),
            secret_access_key: "67890".to_string(),
            session_token: Some("abcde".to_string()),
        };
        assert_eq!(add_or_edit_aws_profile(test_profile.clone()).unwrap(), ());
    
        let details = get_aws_profile_details("test-profile").unwrap();
        assert_eq!(details.access_key_id, "12345");
        assert_eq!(details.secret_access_key, "67890");
        assert_eq!(details.session_token, Some("abcde".to_string()));
    }

    #[test]
    #[serial]
    fn test_add_or_edit_aws_profile() {
        let temp_dir = tempdir().expect("Failed to create a temp directory");    
        let aws_dir = temp_dir.path().join(".aws");
        let credentials_path = aws_dir.join("credentials");
        std::env::set_var("HOME", temp_dir.path());

        // Initial profile addition
        let test_profile = AwsProfile {
            profile_name: "test-profile".to_string(),
            access_key_id: "test_access_key_id".to_string(),
            secret_access_key: "test_secret_access_key".to_string(),
            session_token: None,
        };
        assert_eq!(add_or_edit_aws_profile(test_profile.clone()).unwrap(), ());
    
        // Profile edition
        let edited_profile = AwsProfile {
            profile_name: "test-profile".to_string(),
            access_key_id: "edited_test_access_key_id".to_string(),
            secret_access_key: "edited_test_secret_access_key".to_string(),
            session_token: Some("edited_test_session_token".to_string()),
        };
        assert_eq!(add_or_edit_aws_profile(edited_profile.clone()).unwrap(), ());
        
        // Load the Ini file after the edits
        let mut config = Ini::new();
        config.load(credentials_path.to_str().unwrap()).expect("Failed to load Ini from file after edits");
    
        // Verify the modifications in the Ini file
        assert_eq!(config.get("test-profile", "aws_access_key_id"), Some("edited_test_access_key_id".to_string()), "Edited Access Key ID not present in Ini file");
        assert_eq!(config.get("test-profile", "aws_secret_access_key"), Some("edited_test_secret_access_key".to_string()), "Edited Secret Access Key not present in Ini file");
        assert_eq!(config.get("test-profile", "aws_session_token"), Some("edited_test_session_token".to_string()), "Edited Session Token not present in Ini file");
    }

    #[test]
    #[serial]
    fn test_delete_aws_profile() {
        let temp_dir = tempdir().expect("Failed to create a temp directory");
        std::env::set_var("HOME", temp_dir.path());
    
        // Initial profile addition
        let test_profile = AwsProfile {
            profile_name: "test-profile".to_string(),
            access_key_id: "test_access_key_id".to_string(),
            secret_access_key: "test_secret_access_key".to_string(),
            session_token: None,
        };
        assert_eq!(add_or_edit_aws_profile(test_profile).unwrap(), ());
    
        // Verify the profile is added
        let profiles_before_deletion = list_aws_profiles().unwrap();
        assert!(profiles_before_deletion.contains(&"test-profile".to_string()));
    
        // Execute the function under test
        assert_eq!(delete_aws_profile("test-profile").unwrap(), ());
    
        // Verify the profile is deleted
        let profiles_after_deletion = list_aws_profiles().unwrap();
        assert!(!profiles_after_deletion.contains(&"test-profile".to_string()), "The profile should have been deleted.");
    
        // Optional: Verify by attempting to fetch deleted profile details (should fail)
        let result = get_aws_profile_details("test-profile");
        assert!(result.is_err(), "Fetching details for a deleted profile should fail.");
    }
}
