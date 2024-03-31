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

    let credentials_path = std::path::Path::new(&home_dir).join(".aws/credentials");
    let mut config = Ini::new();

    if credentials_path.exists() {
        // Load existing configuration if the file exists
        match config.load(credentials_path.to_str().unwrap()) {
            Ok(_) => (),
            Err(e) => return Err(format!("Failed to load existing AWS credentials file: {}", e)),
        }
    }

    // Set values for the specified profile
    config.set(&profile.profile_name, "aws_access_key_id", Some(profile.access_key_id.to_string()));
    config.set(&profile.profile_name, "aws_secret_access_key", Some(profile.secret_access_key.to_string()));

    if let Some(token) = profile.session_token {
        config.set(&profile.profile_name, "aws_session_token", Some(token.to_string()));
    } else {
        // Ensure any existing session token is removed if not provided
        config.set(&profile.profile_name, "aws_session_token", None);
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
    use std::io::{Write, Read};
    use tempfile::tempdir;

    #[test]
    fn test_list_aws_profiles() {
        // Set up a temporary directory
        let temp = tempdir().unwrap();
        let cred_path = temp.path().join(".aws/credentials");

        // Ensure the .aws directory exists
        std::fs::create_dir_all(cred_path.parent().unwrap()).unwrap();

        // Write a mock credentials file
        let mut file = std::fs::File::create(&cred_path).unwrap();
        writeln!(file, "[default]\naws_access_key_id=AKIA...").unwrap();
        writeln!(file, "[test-profile]\naws_access_key_id=AKIA...").unwrap();

        // Mock the HOME environment variable
        std::env::set_var("HOME", temp.path());

        // Execute the function under test
        let profiles = list_aws_profiles().unwrap();

        // Verify the results
        assert_eq!(profiles, vec!["default".to_string(), "test-profile".to_string()]);
    }

    #[test]
    fn test_get_aws_profile_details() {
        let temp = tempdir().unwrap();
        let cred_path = temp.path().join(".aws/credentials");
    
        std::fs::create_dir_all(cred_path.parent().unwrap()).unwrap();
    
        let mut file = std::fs::File::create(&cred_path).unwrap();
        writeln!(file, "[test-profile]").unwrap();
        writeln!(file, "aws_access_key_id=test_access_key_id").unwrap();
        writeln!(file, "aws_secret_access_key=test_secret_access_key").unwrap();
        file.flush().unwrap(); // Ensure the file is flushed properly
    
        std::env::set_var("HOME", temp.path());
    
        let details = get_aws_profile_details("test-profile").unwrap();
        assert_eq!(details.access_key_id, "test_access_key_id");
        assert_eq!(details.secret_access_key, "test_secret_access_key");
        assert!(details.session_token.is_none());
    }

    #[test]
    fn test_add_or_edit_aws_profile() {
        let temp = tempdir().unwrap();
        let cred_path = temp.path().join(".aws/credentials");
    
        std::fs::create_dir_all(cred_path.parent().unwrap()).unwrap();
    
        // Mock the HOME environment variable to use the temporary directory
        std::env::set_var("HOME", temp.path());
    
        // Create a test profile to add
        let test_profile = AwsProfile {
            profile_name: "test-profile".to_string(),
            access_key_id: "test_access_key_id".to_string(),
            secret_access_key: "test_secret_access_key".to_string(),
            session_token: None,
        };
    
        // Call the function under test to add the new profile
        let add_result = add_or_edit_aws_profile(test_profile.clone());
        assert!(add_result.is_ok(), "Failed to add or edit AWS profile: {:?}", add_result.err().unwrap());
    
        // Re-read the file contents after adding the profile
        let mut file = std::fs::File::open(&cred_path).expect("Failed to open credentials file after adding profile");
        let mut contents = String::new();
        file.read_to_string(&mut contents).expect("Failed to read credentials file after adding profile");
        assert!(contents.contains("[test-profile]"), "Profile section not found in file after adding");
    
        // Edit the existing profile
        let edited_profile = AwsProfile {
            profile_name: "test-profile".to_string(),
            access_key_id: "new_test_access_key_id".to_string(),
            secret_access_key: "new_test_secret_access_key".to_string(),
            session_token: Some("new_test_session_token".to_string()),
        };
    
        // Call the function under test to edit the existing profile
        let edit_result = add_or_edit_aws_profile(edited_profile);
        assert!(edit_result.is_ok(), "Failed to edit AWS profile");
    
        // Re-read the file contents after editing the profile
        let mut file = std::fs::File::open(&cred_path).expect("Failed to open credentials file after editing profile");
        contents.clear();
        file.read_to_string(&mut contents).expect("Failed to read credentials file after editing profile");
        assert!(contents.contains("[test-profile]"), "Profile section not found in file after editing");
        assert!(contents.contains("aws_access_key_id=new_test_access_key_id"), "Edited Access Key ID not found in file");
        assert!(contents.contains("aws_secret_access_key=new_test_secret_access_key"), "Edited Secret Access Key not found in file");
        assert!(contents.contains("aws_session_token=new_test_session_token"), "Edited Session Token not found in file");
    }
}
