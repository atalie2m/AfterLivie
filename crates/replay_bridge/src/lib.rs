use replay_core::{
    default_sidebar_plan, Diagnostic, DiagnosticCategory, DiagnosticSeverity, SourceRef,
};
use serde::{Deserialize, Serialize};
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::path::PathBuf;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct BridgeResponse<T: Serialize> {
    ok: bool,
    data: Option<T>,
    diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct VersionPayload {
    app_version: &'static str,
    core_version: &'static str,
    bridge_version: &'static str,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ImportPlanRequest {
    comments_path: String,
    #[serde(default = "default_media_width")]
    media_width: u32,
    #[serde(default = "default_media_height")]
    media_height: u32,
    #[serde(default = "default_fps")]
    fps: f64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ImportPlanPayload {
    import_result: replay_core::ImportResult,
    render_plan: replay_core::SemanticRenderPlan,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProbeMediaRequest {
    video_path: String,
}

fn default_media_width() -> u32 {
    1280
}

fn default_media_height() -> u32 {
    720
}

fn default_fps() -> f64 {
    30.0
}

#[no_mangle]
pub extern "C" fn replay_version_json() -> *mut c_char {
    response_json(BridgeResponse {
        ok: true,
        data: Some(VersionPayload {
            app_version: env!("CARGO_PKG_VERSION"),
            core_version: env!("CARGO_PKG_VERSION"),
            bridge_version: env!("CARGO_PKG_VERSION"),
        }),
        diagnostics: vec![],
    })
}

#[no_mangle]
pub extern "C" fn replay_diagnostics_smoke_json() -> *mut c_char {
    response_json(BridgeResponse {
        ok: true,
        data: Some(serde_json::json!({ "message": "bridge smoke ok" })),
        diagnostics: vec![Diagnostic::new(
            DiagnosticSeverity::Info,
            DiagnosticCategory::Project,
            "bridge.smoke",
            "Swift/Rust bridge diagnostics smoke test succeeded.",
        )],
    })
}

#[no_mangle]
pub extern "C" fn replay_import_plan_json(input_json: *const c_char) -> *mut c_char {
    with_input(input_json, |request: ImportPlanRequest| {
        let import_result = replay_importers::import_canonical_json(&request.comments_path);
        let plan = default_sidebar_plan(
            request.media_width,
            request.media_height,
            request.fps,
            vec![import_result.source.clone()],
            import_result.comments.clone(),
        );
        BridgeResponse {
            ok: !has_fatal(&import_result.diagnostics),
            data: Some(ImportPlanPayload {
                import_result: import_result.clone(),
                render_plan: plan,
            }),
            diagnostics: import_result.diagnostics,
        }
    })
}

#[no_mangle]
pub extern "C" fn replay_probe_media_json(input_json: *const c_char) -> *mut c_char {
    with_input(
        input_json,
        |request: ProbeMediaRequest| match replay_media::probe_media(PathBuf::from(
            &request.video_path,
        )) {
            Ok(result) => BridgeResponse {
                ok: !has_fatal(&result.diagnostics),
                data: result.info,
                diagnostics: result.diagnostics,
            },
            Err(error) => BridgeResponse {
                ok: false,
                data: None,
                diagnostics: vec![Diagnostic::new(
                    DiagnosticSeverity::Fatal,
                    DiagnosticCategory::Media,
                    "media.probe_error",
                    error.to_string(),
                )
                .with_source_ref(SourceRef::File {
                    path: request.video_path,
                })],
            },
        },
    )
}

#[no_mangle]
pub extern "C" fn replay_render_preview_json(input_json: *const c_char) -> *mut c_char {
    with_input(input_json, |request: replay_render::RenderRequest| {
        match replay_render::render_preview(&request) {
            Ok(report) => BridgeResponse {
                ok: !has_fatal(&report.diagnostics) && !report.cancelled,
                diagnostics: report.diagnostics.clone(),
                data: Some(report),
            },
            Err(error) => render_error(error),
        }
    })
}

#[no_mangle]
pub extern "C" fn replay_render_export_json(input_json: *const c_char) -> *mut c_char {
    with_input(input_json, |request: replay_render::RenderRequest| {
        match replay_render::render_export(&request) {
            Ok(report) => BridgeResponse {
                ok: !has_fatal(&report.diagnostics) && !report.cancelled,
                diagnostics: report.diagnostics.clone(),
                data: Some(report),
            },
            Err(error) => render_error(error),
        }
    })
}

#[no_mangle]
pub extern "C" fn replay_cancel_render_json() -> *mut c_char {
    replay_render::cancel_render();
    response_json(BridgeResponse {
        ok: true,
        data: Some(serde_json::json!({ "cancelRequested": true })),
        diagnostics: vec![Diagnostic::new(
            DiagnosticSeverity::Info,
            DiagnosticCategory::Export,
            "export.cancel_requested",
            "Render cancellation was requested.",
        )],
    })
}

/// Frees a string allocated by an AfterLivie bridge function.
///
/// # Safety
///
/// `ptr` must either be null or a pointer returned by one of this crate's
/// `replay_*_json` functions. Passing any other pointer, or passing the same
/// pointer more than once, is undefined behavior.
#[no_mangle]
pub unsafe extern "C" fn replay_string_free(ptr: *mut c_char) {
    if !ptr.is_null() {
        drop(CString::from_raw(ptr));
    }
}

fn with_input<I, O>(
    input_json: *const c_char,
    handler: impl FnOnce(I) -> BridgeResponse<O>,
) -> *mut c_char
where
    I: for<'de> Deserialize<'de>,
    O: Serialize,
{
    if input_json.is_null() {
        return response_json(BridgeResponse::<O> {
            ok: false,
            data: None,
            diagnostics: vec![Diagnostic::new(
                DiagnosticSeverity::Fatal,
                DiagnosticCategory::Project,
                "bridge.null_input",
                "Bridge input pointer was null.",
            )],
        });
    }

    let input = unsafe { CStr::from_ptr(input_json) };
    let Ok(text) = input.to_str() else {
        return response_json(BridgeResponse::<O> {
            ok: false,
            data: None,
            diagnostics: vec![Diagnostic::new(
                DiagnosticSeverity::Fatal,
                DiagnosticCategory::Project,
                "bridge.invalid_utf8",
                "Bridge input was not valid UTF-8.",
            )],
        });
    };

    match serde_json::from_str::<I>(text) {
        Ok(request) => response_json(handler(request)),
        Err(error) => response_json(BridgeResponse::<O> {
            ok: false,
            data: None,
            diagnostics: vec![Diagnostic::new(
                DiagnosticSeverity::Fatal,
                DiagnosticCategory::Schema,
                "bridge.invalid_json",
                format!("Bridge input JSON is invalid: {error}"),
            )],
        }),
    }
}

fn response_json<T: Serialize>(value: T) -> *mut c_char {
    let json = serde_json::to_string(&value).unwrap_or_else(|error| {
        format!(
            r#"{{"ok":false,"data":null,"diagnostics":[{{"id":"bridge_serialize_error","severity":"fatal","category":"project","message":"{error}","sourceRef":null,"code":"bridge.serialize_error","actionableHint":null}}]}}"#
        )
    });
    CString::new(json)
        .expect("serialized JSON cannot contain interior null bytes")
        .into_raw()
}

fn has_fatal(diagnostics: &[Diagnostic]) -> bool {
    diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == DiagnosticSeverity::Fatal)
}

fn render_error(error: anyhow::Error) -> BridgeResponse<replay_render::RenderReport> {
    BridgeResponse {
        ok: false,
        data: None,
        diagnostics: vec![Diagnostic::new(
            DiagnosticSeverity::Fatal,
            DiagnosticCategory::Export,
            "export.render_error",
            error.to_string(),
        )],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_response_is_json() {
        let ptr = replay_version_json();
        let text = unsafe { CStr::from_ptr(ptr).to_str().unwrap().to_string() };
        unsafe { replay_string_free(ptr) };
        assert!(text.contains("\"ok\":true"));
    }
}
