# Create the directory first
mkdir -p models/all-MiniLM-L6-v2

# Download the files using curl
curl -L https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2/resolve/main/config.json -o models/all-MiniLM-L6-v2/config.json

curl -L https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2/resolve/main/tokenizer.json -o models/all-MiniLM-L6-v2/tokenizer.json
curl -L https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2/resolve/main/pytorch_model.bin -o models/all-MiniLM-L6-v2/model.ot
