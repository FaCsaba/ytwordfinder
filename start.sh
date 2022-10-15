be () {
    cd backend && cargo build -r && cargo r -r;
}

fe () {
    cd frontend && npm i && npm run build && mv dist/* /var/www/ytwordfinder;
}

be & fe
