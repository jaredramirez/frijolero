check:
    watchexec -c \
        --filter "src/*.rs" \
        -- cargo check

run file="levels/test/level.ldtk" :
    cargo run -- {{file}}

check-example:
    watchexec -c \
        --filter "example/platformer/*.rs" \
        -- cargo check --example platformer

run-example:
    cargo run --example platformer
