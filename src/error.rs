use axum::{
    body::Body,
    http::{Response, StatusCode},
    response::IntoResponse,
};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("invalid mac address")]
    InvalidMacAddress,

    #[error("invalid request")]
    InvalidRequest,

    #[error("failed to send magic packet")]
    WakeOnLan(#[from] std::io::Error),

    #[error("unknown machine")]
    UnknownMachine,

    #[error("failed to parse machine mapping: {0}")]
    FailedToParseMachineMapping(String),

    #[error("failed to list names")]
    FailedToListNames,

    #[error("failed to get mac address")]
    FailedToGetMacAddress,
}

impl IntoResponse for Error {
    fn into_response(self) -> Response<Body> {
        Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(self.to_string().into())
            .unwrap()
    }
}
