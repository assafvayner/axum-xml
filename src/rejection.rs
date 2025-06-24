use axum_core::extract::rejection::BytesRejection;
use http::StatusCode;
use thiserror::Error;

use crate::IntoResponse;

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
    fn into_response(self) -> crate::Response {
        match self {
            e @ XmlRejection::InvalidXMLBody(_) => {
                (StatusCode::UNPROCESSABLE_ENTITY, e.to_string()).into_response()
            }
            e @ XmlRejection::MissingXMLContentType => {
                (StatusCode::UNSUPPORTED_MEDIA_TYPE, e.to_string()).into_response()
            }
            XmlRejection::BytesRejection(e) => e.into_response(),
        }
    }
}
