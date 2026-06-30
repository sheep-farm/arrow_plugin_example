use hayashi_plugin_sdk::{hayashi_fn, hayashi_plugin};
use hayashi_plugin_sdk::arrow::array::{Array, ArrayRef, Float64Array, BooleanArray, Int64Array, StringArray};
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

/// Demonstração de processamento zero-copy de um DataFrame completo contendo tipos mistos
/// (números, booleanos e strings).
///
/// Recebe um DataFrame contendo as colunas "id" (Int64), "active" (Boolean) e "label" (Utf8),
/// gera as novas colunas:
/// - "is_valid" = active && id > 15
/// - "message" = "ID [id]: [label]" (String)
///
/// Retorna o DataFrame estendido de forma zero-copy.
///
/// Exemplo de uso no Hayashi:
/// let df_mixed = tp::process_mixed_dataframe(df)
#[hayashi_fn]
pub fn process_mixed_dataframe(arr: ArrayRef) -> Result<ArrayRef, String> {
    use hayashi_plugin_sdk::arrow::array::StructArray;
    use hayashi_plugin_sdk::arrow::datatypes::{Field, Fields, DataType};

    let struct_arr = arr.as_any()
        .downcast_ref::<StructArray>()
        .ok_or_else(|| "Esperado um StructArray correspondente ao DataFrame".to_string())?;

    let id_array = struct_arr.column_by_name("id")
        .ok_or_else(|| "Coluna 'id' não encontrada".to_string())?
        .as_any()
        .downcast_ref::<Int64Array>()
        .ok_or_else(|| "Coluna 'id' deve ser do tipo Int64".to_string())?;

    let active_array = struct_arr.column_by_name("active")
        .ok_or_else(|| "Coluna 'active' não encontrada".to_string())?
        .as_any()
        .downcast_ref::<BooleanArray>()
        .ok_or_else(|| "Coluna 'active' deve ser do tipo Boolean".to_string())?;

    let label_array = struct_arr.column_by_name("label")
        .ok_or_else(|| "Coluna 'label' não encontrada".to_string())?
        .as_any()
        .downcast_ref::<StringArray>()
        .ok_or_else(|| "Coluna 'label' deve ser do tipo String (Utf8)".to_string())?;

    let len = struct_arr.len();
    let mut is_valid_vec = Vec::with_capacity(len);
    let mut message_vec = Vec::with_capacity(len);

    for i in 0..len {
        let id = id_array.value(i);
        let active = active_array.value(i);
        let label = label_array.value(i);

        is_valid_vec.push(active && id > 15);
        message_vec.push(format!("ID {}: {}", id, label));
    }

    let is_valid_array = Arc::new(BooleanArray::from(is_valid_vec)) as ArrayRef;
    let message_array = Arc::new(StringArray::from(message_vec)) as ArrayRef;

    // Constrói o StructArray de saída com todas as colunas
    let fields = vec![
        Field::new("id", DataType::Int64, true),
        Field::new("active", DataType::Boolean, true),
        Field::new("label", DataType::Utf8, true),
        Field::new("is_valid", DataType::Boolean, true),
        Field::new("message", DataType::Utf8, true),
    ];
    let arrays = vec![
        Arc::new(id_array.clone()) as ArrayRef,
        Arc::new(active_array.clone()) as ArrayRef,
        Arc::new(label_array.clone()) as ArrayRef,
        is_valid_array,
        message_array,
    ];

    let out_struct = StructArray::try_new(Fields::from(fields), arrays, None)
        .map_err(|e| format!("Falha ao criar o StructArray de saída: {e}"))?;

    Ok(Arc::new(out_struct) as ArrayRef)
}
