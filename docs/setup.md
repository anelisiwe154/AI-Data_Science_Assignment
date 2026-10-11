# Setup (Windows)

1. Install Rust and Cargo.
2. Install Ollama and ensure the Ollama app/service is running.
3. In PowerShell, run:

   ```powershell
   ollama pull llama3.2:3b
   ```

4. Open PowerShell in this project directory and run:

   ```powershell
   cargo run
   ```

5. Keep the terminal open and open http://127.0.0.1:7879.

If the server reports that it cannot bind to port 7879, close the previous instance or identify the process already using that port. If Ollama is unavailable, general prospectus questions display a model error rather than silently using the original extractive answer path. APS calculations and structured course predictions do not require Ollama.

To use another installed model, set `OLLAMA_MODEL` before starting the server:

```powershell
$env:OLLAMA_MODEL = "llama3.2:3b"
cargo run
```
