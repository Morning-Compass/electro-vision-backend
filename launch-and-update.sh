clear

echo "Starting launch-and-update.sh script..."

echo "Cargo building..."
# water bucket - --release
cargo build --release

echo "Clearing migrations directory..."
rm -rf ./migrations

echo "Composing down ..."
docker compose down -v

echo "Composing up ..."
docker compose up --build
