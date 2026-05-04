#pragma once

#ifdef __cplusplus
extern "C" {
#endif

char *replay_version_json(void);
char *replay_diagnostics_smoke_json(void);
char *replay_import_plan_json(const char *input_json);
char *replay_probe_media_json(const char *input_json);
char *replay_render_preview_json(const char *input_json);
char *replay_render_export_json(const char *input_json);
char *replay_cancel_render_json(void);
void replay_string_free(char *ptr);

#ifdef __cplusplus
}
#endif
