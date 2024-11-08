#!/bin/bash
export RUST_LOG=debug
export RUST_BACKTRACE=1

cd /home/chat/matcher
exec ./target/release/matcher --server
