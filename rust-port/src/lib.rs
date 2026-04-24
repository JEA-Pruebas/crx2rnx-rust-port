#[derive(Debug)]
pub struct CrxError {
    pub message: String,
}

impl From<&str> for CrxError {
    fn from(msg: &str) -> Self {
        Self {
            message: msg.to_string(),
        }
    }
}

pub fn decompress_crinex(_input: &str) -> Result<String, CrxError> {
    // TODO: implementar descompresión Hatanaka (CRINEX 1.0)
    Err("No implementado".into())
}