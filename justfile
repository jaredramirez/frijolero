check:
    watchexec -c \
        --filter "src/*.rs" \
        -- cargo check

run file="levels/test/level.ldtk" :
    cargo run -- --ldtk-file="{{file}}"

run-dev file="levels/test/level.ldtk" :
    cargo run -- --ldtk-file="{{file}}" --dev

check-example:
    watchexec -c \
        --filter "example/platformer/*.rs" \
        -- cargo check --example platformer

run-example:
    cargo run --example platformer
