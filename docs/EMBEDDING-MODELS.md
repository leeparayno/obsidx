# Local ONNX Embedding Models (ORT)

This doc compares popular local embedding models with ONNX artifacts suitable for ONNX Runtime (ORT): MiniLM, E5, BGE, GTE, and all‑mpnet. Focus: dimensions, size, speed, quality, license, and recommended use.

## Quick recommendations
- **Small + fast**: `all-MiniLM-L6-v2`, `gte-small`, `bge-small-en-v1.5` — best for lightweight CPU use, low latency, and larger vector DBs.
- **Balanced quality**: `e5-base-v2` — strong retrieval quality at 768 dims, moderate size.
- **High quality (older baseline)**: `all-mpnet-base-v2` — higher quality than MiniLM but slower/larger.

## Model comparison (ORT‑friendly ONNX)

| Model | Dim | Size (model card) | License | ONNX availability | Notes / Recommended use |
|---|---:|---:|---|---|---|
| **sentence-transformers/all-MiniLM-L6-v2** | 384 | — | Apache‑2.0 | Yes (ONNX tag + onnx/ files) | Very fast, small vectors; great for lightweight semantic search or clustering. |
| **intfloat/e5-base-v2** | 768 | 0.44 GB | MIT | Yes (ONNX tag) | Strong retrieval quality; use “query: … / passage: …” prefixes. |
| **BAAI/bge-small-en-v1.5** | 384 | — | MIT | Yes (ONNX tag) | Strong small model for English retrieval; low latency. |
| **thenlper/gte-small** | 384 | 0.07 GB | MIT | Yes (ONNX tag) | Excellent small model; competitive MTEB performance for size. |
| **sentence-transformers/all-mpnet-base-v2** | 768 | — | Apache‑2.0 | Yes (ONNX tag + onnx/ files) | Classic strong baseline; higher quality than MiniLM, slower/larger. |

> **Note on size**: Where model card provides size (GB), it’s listed. For others, size is not explicitly stated in the HF model card; use size category (small/base) and ONNX file sizes for practical estimation.

## Quality and performance notes

### MiniLM (all‑MiniLM‑L6‑v2)
- **Quality**: Solid for general semantic search, clustering.
- **Speed**: Very fast on CPU (small model, 384‑dim vectors).
- **Tradeoff**: Lower absolute retrieval quality vs. larger models.

### E5 (e5‑base‑v2)
- **Quality**: Strong retrieval performance; widely used in RAG.
- **Speed**: Moderate; 768‑dim vectors increase memory and index size.
- **Tradeoff**: Larger index and compute vs. small models.

### BGE (bge‑small‑en‑v1.5)
- **Quality**: Strong small‑model retrieval for English.
- **Speed**: Fast (small model, 384‑dim).
- **Tradeoff**: English‑focused (use `bge-m3` or larger variants for multilingual).

### GTE (gte‑small)
- **Quality**: Strong MTEB results for small size; competitive with larger models on some tasks.
- **Speed**: Very fast, very small.
- **Tradeoff**: Lower than base/large models in some retrieval tasks.

### MPNet (all‑mpnet‑base‑v2)
- **Quality**: Strong traditional baseline.
- **Speed**: Slower than MiniLM due to base model size.
- **Tradeoff**: Larger vectors and latency vs. modern small models.

## Detailed sources

### Dimensions
- all‑MiniLM‑L6‑v2 maps to **384‑dim** vectors.  
  Source: HF model card (“maps… to a **384 dimensional** dense vector space”).  
  https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2

- all‑mpnet‑base‑v2 maps to **768‑dim** vectors.  
  Source: HF model card (“maps… to a **768 dimensional** dense vector space”).  
  https://huggingface.co/sentence-transformers/all-mpnet-base-v2

- e5‑base‑v2 has embedding size **768**.  
  Source: HF model card (“embedding size is 768”).  
  https://huggingface.co/intfloat/e5-base-v2

- gte‑small dim **384** and model size **0.07 GB**; e5‑base‑v2 size **0.44 GB**.  
  Source: GTE model card table (Model Size & Dimension).  
  https://huggingface.co/thenlper/gte-small

- bge‑small‑en‑v1.5 hidden size **384** (config.json).  
  Source: model config (`hidden_size: 384`).  
  https://huggingface.co/BAAI/bge-small-en-v1.5/raw/main/config.json

### Licenses (HF API tags)
- all‑MiniLM‑L6‑v2: **Apache‑2.0**  
  https://huggingface.co/api/models/sentence-transformers/all-MiniLM-L6-v2

- all‑mpnet‑base‑v2: **Apache‑2.0**  
  https://huggingface.co/api/models/sentence-transformers/all-mpnet-base-v2

- e5‑base‑v2: **MIT**  
  https://huggingface.co/api/models/intfloat/e5-base-v2

- gte‑small: **MIT**  
  https://huggingface.co/api/models/thenlper/gte-small

- bge‑small‑en‑v1.5: **MIT**  
  https://huggingface.co/api/models/BAAI/bge-small-en-v1.5

### ONNX availability
- All models listed include **ONNX** artifacts or tags on Hugging Face (model tag includes `onnx`).  
  Example: all‑MiniLM‑L6‑v2 has multiple `onnx/` files in repo.  
  https://huggingface.co/api/models/sentence-transformers/all-MiniLM-L6-v2

---

If you want, add a short section on **quantized ONNX variants** (INT8) for MiniLM/Mpnet, or include **recommended pooling** (mean vs CLS) per model.
