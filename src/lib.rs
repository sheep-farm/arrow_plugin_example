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

/// Demonstração de processamento zero-copy de um DataFrame completo.
/// Recebe um DataFrame inteiro como um StructArray via Arrow FFI,
/// lê as colunas "x" e "y", calcula a nova coluna "z" = x * y,
/// e retorna um novo DataFrame (StructArray) contendo "x", "y" e "z".
///
/// Exemplo de uso no Hayashi:
/// let df_new = tp::process_dataframe(df)
#[hayashi_fn]
pub fn process_dataframe(arr: ArrayRef) -> Result<ArrayRef, String> {
    use hayashi_plugin_sdk::arrow::array::StructArray;
    use hayashi_plugin_sdk::arrow::datatypes::{Field, Fields, DataType};

    let struct_arr = arr.as_any()
        .downcast_ref::<StructArray>()
        .ok_or_else(|| "Esperado um StructArray correspondente ao DataFrame".to_string())?;
        
    let x_array = struct_arr.column_by_name("x")
        .ok_or_else(|| "Coluna 'x' não encontrada no DataFrame".to_string())?
        .as_any()
        .downcast_ref::<Float64Array>()
        .ok_or_else(|| "Coluna 'x' deve ser do tipo Float64".to_string())?;
        
    let y_array = struct_arr.column_by_name("y")
        .ok_or_else(|| "Coluna 'y' não encontrada no DataFrame".to_string())?
        .as_any()
        .downcast_ref::<Float64Array>()
        .ok_or_else(|| "Coluna 'y' deve ser do tipo Float64".to_string())?;
        
    let z_values: Vec<f64> = x_array.values().iter()
        .zip(y_array.values().iter())
        .map(|(&x, &y)| x * y)
        .collect();
        
    let z_array = Arc::new(Float64Array::from(z_values)) as ArrayRef;
    
    // Constrói um novo StructArray de retorno
    let fields = vec![
        Field::new("x", DataType::Float64, true),
        Field::new("y", DataType::Float64, true),
        Field::new("z", DataType::Float64, true),
    ];
    let arrays = vec![
        Arc::new(x_array.clone()) as ArrayRef,
        Arc::new(y_array.clone()) as ArrayRef,
        z_array,
    ];
    
    let out_struct = StructArray::try_new(Fields::from(fields), arrays, None)
        .map_err(|e| format!("Falha ao criar o StructArray de saída: {e}"))?;
        
    Ok(Arc::new(out_struct) as ArrayRef)
}
