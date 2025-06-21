

# Display this message
@default:
  just --list

# Run the dev build
@dev $RUST_LOG="debug" $RUST_BACKTRACE="1":
  cargo run


maudVersion := "v0.27.0"
update-ast:
  cd src/vendor && curl -O https://raw.githubusercontent.com/lambda-fairy/maud/refs/tags/{{maudVersion}}/maud_macros/src/ast.rs
