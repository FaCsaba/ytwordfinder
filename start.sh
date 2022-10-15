be () {
    cd backend && cargo build -r && cargo r -r;
}

fe () {
    cd frontend && npm i && npm run build;
}

be & fe
