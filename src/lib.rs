use hayashi_plugin_sdk::{hayashi_fn, hayashi_plugin};
use hayashi_plugin_sdk::arrow::array::{ArrayRef, Float64Array};
use std::sync::Arc;

// Expõe os desalocadores da ABI C usados pelo Hayashi para limpar memória
hayashi_plugin!();

/// Multiplica todos os valores de uma coluna numérica de ponto flutuante por um fator.
/// A coluna é passada de forma transparente via ponteiro FFI do Apache Arrow (Zero-Copy),
/// e o resultado é devolvido também via ponteiro de array Arrow sem qualquer cópia/serialização JSON dos dados.
///
/// Exemplo de uso no Hayashi:
/// let df = load("dados.csv")
/// import("arrow_plugin_example", as=tp)
/// generate df x_scaled = tp::scale_column(df["x"], 2.5)
#[hayashi_fn]
pub fn scale_column(arr: ArrayRef, factor: f64) -> Result<ArrayRef, String> {
    let float_arr = arr.as_any()
        .downcast_ref::<Float64Array>()
        .ok_or_else(|| "Esperado uma coluna do tipo Float64Array".to_string())?;
    
    // Processamento e vetorização direta em Rust
    let scaled_values: Vec<f64> = float_arr.values()
        .iter()
        .map(|&v| v * factor)
        .collect();
    
    let new_arr = Float64Array::from(scaled_values);
    Ok(Arc::new(new_arr) as ArrayRef)
}

/// Realiza o somatório de uma coluna numérica lendo os valores diretamente
/// do array em memória compartilhada via FFI do Apache Arrow (Zero-Copy).
/// Retorna um valor numérico escalar puro.
///
/// Exemplo de uso no Hayashi:
/// let total = tp::sum_column(df["x"])
#[hayashi_fn]
pub fn sum_column(arr: ArrayRef) -> Result<f64, String> {
    let float_arr = arr.as_any()
        .downcast_ref::<Float64Array>()
        .ok_or_else(|| "Esperado uma coluna do tipo Float64Array".to_string())?;
    
    let total: f64 = float_arr.values().iter().sum();
    Ok(total)
}

/// Exemplo tradicional usando serialização padrão (JSON C String).
/// Recebe uma lista ordinária como argumento (convertida para Vec no guest)
/// e retorna a soma dos valores.
///
/// Exemplo de uso no Hayashi:
/// let total = tp::sum_column_vector(df["x"])
#[hayashi_fn]
pub fn sum_column_vector(values: Vec<f64>) -> f64 {
    values.iter().sum()
}
