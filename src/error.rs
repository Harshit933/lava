use std::fmt;

use axum::response::{IntoResponse, Response};

#[derive(Debug)]
pub enum LavaErrors {
    NoContractID,
    FailedToUpdateBtcBalance,
    FailedToUpdateSolBalance,
}

impl std::error::Error for LavaErrors {}

impl fmt::Display for LavaErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LavaErrors::NoContractID => write!(f, "No contract ID found"),
            LavaErrors::FailedToUpdateBtcBalance => write!(f, "Failed to update BTC balance"),
            LavaErrors::FailedToUpdateSolBalance => write!(f, "Failed to update Sol balance")
        }
    }
}

impl IntoResponse for LavaErrors {
    fn into_response(self) -> Response {
        match self {
            LavaErrors::NoContractID => "No contract ID found".into_response(),
            LavaErrors::FailedToUpdateBtcBalance => "Failed to update BTC balance".into_response(),
            LavaErrors::FailedToUpdateSolBalance => "Failed to update Sol balance".into_response(),
        }
    }
}