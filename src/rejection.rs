use axum_core::extract::rejection::BytesRejection;
use axum_core::response::{IntoResponse, Response};
use http::StatusCode;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum XmlRejection {
    #[error("Failed to parse the request body as XML")]
    InvalidXMLBody(#[from] quick_xml::DeError),
    #[error("Expected request with `Content-Type: application/xml`")]
    MissingXMLContentType,
    #[error("{0}")]
    BytesRejection(#[from] BytesRejection),
}

impl IntoResponse for XmlRejection {
    fn into_response(self) -> Response {
        match self {
            e @ Self::InvalidXMLBody(_) => {
                (StatusCode::UNPROCESSABLE_ENTITY, e.to_string()).into_response()
            }
            e @ Self::MissingXMLContentType => {
                (StatusCode::UNSUPPORTED_MEDIA_TYPE, e.to_string()).into_response()
            }
            Self::BytesRejection(e) => e.into_response(),
        }
    }
}
