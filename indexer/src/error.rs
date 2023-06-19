use hub_core::reqwest::StatusCode;
use poem::{error::ResponseError, Body, Response};

#[derive(Debug, thiserror::Error)]
#[error("errors")]
pub enum Error {
    MissingHeader,
    InvalidUtf8,
    InvalidHexadecimal,
}

impl ResponseError for Error {
    fn status(&self) -> StatusCode {
        match self {
            Self::MissingHeader => StatusCode::NOT_FOUND,
            _ => StatusCode::BAD_REQUEST,
        }
    }

    fn as_response(&self) -> Response {
        let message = match self {
            Self::MissingHeader => "X-Alchemy-Signature header is missing",
            Self::InvalidUtf8 => "X-Alchemy-Signature header is not valid UTF-8",
            Self::InvalidHexadecimal => "X-Alchemy-Signature header is not valid hexadecimal",
        };

        Response::builder()
            .status(self.status())
            .body(Body::from_string(message.to_string()))
    }
}
