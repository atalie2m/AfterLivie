set shell := ["bash", "-eu", "-o", "pipefail", "-c"]

default:
  just --list

doctor:
  cargo run -p replay_cli -- doctor

fmt:
  cargo fmt --all

fmt-check:
  cargo fmt --all -- --check

clippy:
  cargo clippy --workspace --all-targets -- -D warnings

test:
  cargo nextest run

test-cargo:
  cargo test --workspace

sample-plan:
  cargo run -p replay_cli -- import-plan --comments fixtures/comments/basic_en.json --media-width 1280 --media-height 720 --media-duration-ms 30000 --out fixtures/render_plans/sidebar_basic.json

sample-overlay:
  cargo run -p replay_cli -- overlay-frame --plan fixtures/render_plans/sidebar_basic.json --time-ms 10000 --out fixtures/golden_images/sidebar_basic.png
