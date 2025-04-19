clear

echo "Starting launch-and-update.sh script..."

echo "Clearing migrations directory..."
rm -rf ./migrations

echo "Composing down ..."
docker compose down -v

echo "Composing up ..."
docker compose up --build
