docker run --rm -v `pwd`:/rust_tmp -v $HOME/.cargo/registry:/usr/local/cargo/registry -w /rust_tmp rust_dev cargo build --release && \
docker build . -t cr.local.fzxu.me:5000/xnote:0.73 && \
docker push cr.local.fzxu.me:5000/xnote:0.73