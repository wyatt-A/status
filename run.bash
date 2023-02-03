#!/usr/bin/bash

TARGET_CC=x86_64-linux-musl-gcc cargo build --release --target x86_64-unknown-linux-musl &&
cargo build --release &&
scp /Users/Wyatt/IdeaProjects/status/target/x86_64-unknown-linux-musl/release/status civmcluster1:/home/wa41/.pipe_status/ &&
scp /Users/Wyatt/IdeaProjects/status/target/release/status delos:/Users/wyatt/.pipe_status/ &&
cargo run N60217 diffusion_calc_nlsam -c=/Users/Wyatt/IdeaProjects/status/pipe_configs