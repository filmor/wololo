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

    #[error("failed to parse machine: {0}")]
    FailedToParseMachine(String),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response<Body> {
        Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(self.to_string().into())
            .unwrap()
    }
}
