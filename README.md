# Create the directory first
mkdir -p models/all-MiniLM-L6-v2

# Download the files using curl
curl -L https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2/resolve/main/config.json -o models/all-MiniLM-L6-v2/config.json

curl -L https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2/resolve/main/tokenizer.json -o models/all-MiniLM-L6-v2/tokenizer.json
curl -L https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2/resolve/main/pytorch_model.bin -o models/all-MiniLM-L6-v2/model.ot

OR

# Or for multi lingual
mkdir -p models/multilingual-MiniLM
curl -L https://huggingface.co/sentence-transformers/paraphrase-multilingual-MiniLM-L12-v2/resolve/main/config.json -o models/multilingual-MiniLM/config.json
curl -L https://huggingface.co/sentence-transformers/paraphrase-multilingual-MiniLM-L12-v2/resolve/main/tokenizer.json -o models/multilingual-MiniLM/tokenizer.json
curl -L https://huggingface.co/sentence-transformers/paraphrase-multilingual-MiniLM-L12-v2/resolve/main/pytorch_model.bin -o models/multilingual-MiniLM/model.ot

cargo run -- --query "execute la meilleure analyse de gpecs"

cargo run -- --reload --debug --all --query "fait une analyse de gpecs"


# Show help
matcher --help
./target/debug/matcher --help

# Show version
matcher --version

# Basic query => EN !!!
matcher -q "run analysis"

# Verbose output with multiple results
matcher -q "execute une analyse de gpecs"


# Test French queries => FR !!!
cargo run -- --query "Pourriez-vous lancer l'analyse de toto" --language fr

# Test with database reload
cargo run -- --reload --query "Je voudrais effectuer un calcul de titi" --language fr


./target/debug/matcher --query "envoie le document par email à fawzan@gmail.com"


./target/debug/matcher --server
