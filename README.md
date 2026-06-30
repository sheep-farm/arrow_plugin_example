# arrow_plugin_example

Este é um repositório de exemplo demonstrando como criar um plugin nativo de alta performance para a linguagem **Hayashi** utilizando **Apache Arrow FFI (Zero-Copy)**.

## O que é Zero-Copy FFI?

Diferente do mecanismo padrão de plugins nativos que troca argumentos usando strings JSON serializadas via C ABI, o Hayashi suporta a passagem direta de ponteiros de memória de colunas de DataFrames baseados nas especificações da FFI do **Apache Arrow**.

Isso significa que:
1. O host Hayashi passa um ponteiro de struct `FFI_ArrowArray` do Arrow contendo as referências diretas dos buffers de dados.
2. O plugin (guest) reconstrói o array estatístico a partir desse endereço e o lê sem realizar nenhuma cópia de dados na memória.
3. Se o plugin retornar uma nova coluna, ele também devolve ponteiros de structs FFI Arrow de forma que o host possa utilizá-los instantaneamente de forma zero-copy.

Esse padrão elimina completamente o overhead de serialização/deserialização para Big Data, tornando as extensões tão rápidas quanto o próprio núcleo da engine.

## Estrutura do Projeto

* `Cargo.toml`: Configuração do crate como biblioteca do tipo `cdylib` dependendo do `hayashi-plugin-sdk`.
* `src/lib.rs`: Implementação das funções expondo:
  * `scale_column(arr: ArrayRef, factor: f64) -> Result<ArrayRef, String>`: Demonstração de recebimento e retorno usando Apache Arrow FFI de forma zero-copy.
  * `sum_column(arr: ArrayRef) -> Result<f64, String>`: Demonstração de leitura direta em vetor mapeado do Arrow retornando um escalar.
  * `process_dataframe(arr: ArrayRef) -> Result<ArrayRef, String>`: Processamento de um DataFrame completo de forma zero-copy através de um `StructArray` contendo múltiplas colunas.
  * `sum_column_vector(values: Vec<f64>) -> f64`: Exemplo de comparação usando a serialização tradicional baseada em JSON.

## Como Compilar o Plugin

Para compilar e gerar a biblioteca dinâmica correspondente ao seu sistema operacional:

```bash
cargo build --release
```

Os binários serão gerados na pasta `target/release/`:
* Linux: `libarrow_plugin_example.so`
* macOS: `libarrow_plugin_example.dylib`
* Windows: `arrow_plugin_example.dll`

## Como Usar no Hayashi

Escreva um script `.hay` (ex: `script.hay`):

```text
// Carrega os dados
let df = load("dados.csv")

// Importa a biblioteca compilada do plugin
import("caminho/para/target/release/libarrow_plugin_example", as=tp)

// 1. Processamento Zero-Copy de Coluna Individual (Array FFI)
generate df x_scaled = tp::scale_column(df["x"], 2.5)

// 2. Leitura Zero-Copy retornando valor escalar
let total = tp::sum_column(df["x"])
print("Soma total: ", total)

// 3. Processamento Zero-Copy de DataFrame Completo (StructArray FFI)
let df_new = tp::process_dataframe(df)
display df_new
```
