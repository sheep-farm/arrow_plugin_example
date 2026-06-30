# arrow_plugin_example

This is an example repository demonstrating how to create high-performance native plugins for the **Hayashi** language using **Apache Arrow FFI (Zero-Copy)**.

## What is Zero-Copy FFI?

Unlike the default native plugin mechanism that exchanges arguments using JSON serialized strings via the C ABI, Hayashi supports direct exchange of DataFrame columns and tables via the **Apache Arrow FFI** specification.

This means that:
1. The Hayashi host passes an Arrow `FFI_ArrowArray` struct pointer containing direct references to data buffers.
2. The guest plugin reconstructs the array from this memory address and reads/writes it without any data copy.
3. If the plugin returns a new array or table, it returns Arrow FFI pointers so the host can import them instantly.

This pattern completely eliminates serialization/deserialization overhead for Big Data, making plugins run as fast as the engine core itself.

## Project Structure

* `Cargo.toml`: Crate configuration as a `cdylib` library depending on `hayashi-plugin-sdk`.
* `src/lib.rs`: Implementation of FFI functions:
  * `scale_column(arr: ArrayRef, factor: f64) -> Result<ArrayRef, String>`: Scales an array by a factor zero-copy.
  * `sum_column(arr: ArrayRef) -> Result<f64, String>`: Reads an Arrow array directly and returns a scalar sum.
  * `process_dataframe(arr: ArrayRef) -> Result<ArrayRef, String>`: Processes a numeric DataFrame using a single Arrow `StructArray`.
  * `process_mixed_dataframe(arr: ArrayRef) -> Result<ArrayRef, String>`: Processes a heterogeneous DataFrame containing mixed types (`Int64`, `Boolean`, and `Utf8`) zero-copy.

## How to Install

Install the package directly from GitHub using the Hayashi CLI:

```bash
hay install sheep-farm/arrow_plugin_example
```

This will download the native dynamic library pre-compiled by CI/CD and verify its GitHub Artifact Attestation for cryptographic build provenance.

## How to Use in Hayashi

After installation, import the package in your `.hay` script:

```text
// Load data
let df = load("data.csv")

// Import the installed Arrow plugin
import("sheep-farm/arrow_plugin_example", as=tp)

// 1. Zero-Copy column processing (Array FFI)
generate df x_scaled = tp::scale_column(df["x"], 2.5)

// 2. Zero-Copy column aggregation returning a scalar
let total = tp::sum_column(df["x"])
print("Total sum: ", total)

// 3. Zero-Copy table processing (StructArray FFI)
let df_new = tp::process_dataframe(df)
display df_new

// 4. Zero-Copy heterogeneous table processing (StructArray FFI with mixed types)
let df_mixed_new = tp::process_mixed_dataframe(df)
display df_mixed_new
```
