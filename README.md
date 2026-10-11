# AI-Data_Science_Assignment

This project retains the original APS calculator, course matching, and structured programme features. General prospectus questions use retrieval-augmented generation: the app retrieves prospectus chunks, sends the retrieved context to a local Ollama model, and appends source citations from the chunk metadata.

## Requirements
- Rust toolchain (Cargo)
- Ollama installed and running
- Model `llama3.2:3b` downloaded

## Setup (Windows PowerShell)
```powershell
ollama pull llama3.2:3b
cargo run
```
Then open http://127.0.0.1:7879. Keep the terminal open.

To use a different local model, set `OLLAMA_MODEL`, for example:
```powershell
$env:OLLAMA_MODEL="llama3.2:3b"
cargo run
```

## RAG flow
1. Accept a natural-language question.
2. Retrieve relevant prospectus passages with BM25-style ranking.
3. Generate a grounded answer with Ollama.
4. Attach source document, section, and page citations from the retrieved chunks.

APS calculation and course prediction continue to use the original structured logic. Ollama is required for ordinary generative prospectus answers; if it is unavailable, the app displays an error instead of silently switching to extractive text assembly.

## Citations
`assets/prospectus_pages.txt` is a page-marked extraction. Page numbers are cited only when present in the indexed data; fallback DOCX/plain-text sources may not contain page markers.
