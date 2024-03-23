// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use aws_config::meta::region::RegionProviderChain;
use aws_config::profile::ProfileFileCredentialsProvider;
use aws_sdk_sts::Client as StsClient;

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
            check_aws_identity
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
