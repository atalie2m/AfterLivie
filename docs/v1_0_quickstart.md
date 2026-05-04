# v1.0 Quickstart

1. Enter the development environment with `nix develop`.
2. Validate tools with `nix run .#doctor`.
3. Generate a plan:

   ```sh
   cargo run -p replay_cli -- import-plan \
     --comments fixtures/comments/basic_en.json \
     --media-width 1280 \
     --media-height 720 \
     --out fixtures/render_plans/sidebar_basic.json
   ```

4. Render an overlay frame:

   ```sh
   cargo run -p replay_cli -- overlay-frame \
     --plan fixtures/render_plans/sidebar_basic.json \
     --time-ms 10000 \
     --out fixtures/golden_images/sidebar_basic.png
   ```

5. Probe and export with your own local media:

   ```sh
   cargo run -p replay_cli -- probe-media --video /path/to/video.mp4
   cargo run -p replay_cli -- preview-export \
     --video /path/to/video.mp4 \
     --comments fixtures/comments/basic_en.json \
     --out /tmp/afterlivie-preview.mp4
   ```

The committed media fixture directory intentionally excludes video files until
small redistributable samples are selected.
