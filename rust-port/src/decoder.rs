use crate::error::CrxError;
use crate::native_wrapper::decompress_crinex_native;

pub fn decompress_crinex(input: &str) -> Result<String, CrxError> {
    // Mantener backend nativo de referencia por ahora para garantizar
    // compatibilidad completa con el output esperado.
    decompress_crinex_native(input)
}
