
use h3::client;

pub struct Http3Client {
    pub cert: String,
    pub keys: String,
}

impl Http3Client {
    pub async fn new(cert: String, keys: String) {
        // create http3 client using h3 client
    }

    pub async fn send_request(endpoint: String) {
        // send http3 request to the endpoint and return response
    }
}

