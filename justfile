
# Display this message
@default:
  just --list

# Run the dev build
@dev $RUST_LOG="debug" $RUST_BACKTRACE="1":
  cargo run
