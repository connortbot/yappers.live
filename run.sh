if [ "$1" = "--container" ]; then
    cd wukong
    docker compose up --build
else
    cd wukong
    cargo watch -x run
fi