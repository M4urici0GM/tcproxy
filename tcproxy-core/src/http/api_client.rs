use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::Result;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartChallengeResponse {
    #[serde()]
    challenge_id: Uuid
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartChallengeRequest {
    callback_url: String,
    nonce: u32,
}

impl StartChallengeRequest {
    pub fn new(callback_uri: &str, nonce: &u32) -> Self {
        Self {
            callback_url: String::from(callback_uri),
            nonce: *nonce,
        }
    }
}

impl StartChallengeResponse {
    pub fn challenge_id(&self) -> &Uuid {
        &self.challenge_id
    }
}

pub async fn start_challenge(request: &StartChallengeRequest) -> Result<StartChallengeResponse> {
    let client = reqwest::Client::new();
    let request = client
        .post("http://localhost:5066/v1/auth/start-challenge")
        .json(&request);

    let response = request
        .send()
        .await?
        .json::<StartChallengeResponse>()
        .await?;

    Ok(response)
}