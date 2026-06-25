#![deny(clippy::unwrap_used)]

use hp41_core::CalcState;
use std::ffi::{c_char, CStr, CString};
#[cfg(test)]
use std::fs;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub struct Calculator {
    state: CalcState,
    state_path: PathBuf,
    cards_path: Option<PathBuf>,
    last_error: Option<String>,
    last_result: Option<hp41_app::AppResult>,
}

/// Independent thread-safe cancellation capability. It never aliases mutable
/// calculator state; both sides share only hp41-core's Arc<AtomicBool>.
pub struct CancellationHandle {
    signal: Arc<AtomicBool>,
}

impl Calculator {
    fn load(path: PathBuf) -> Self {
        let state = hp41_app::load_state(&path).unwrap_or_default();
        Self {
            state,
            state_path: path,
            cards_path: hp41_app::cards_dir(),
            last_error: None,
            last_result: None,
        }
    }

    fn save(&self) -> Result<(), String> {
        hp41_app::save_state(&self.state_path, &self.state).map_err(|error| error.to_string())
    }

    fn drain_card_op(&mut self) -> Result<(), String> {
        if self.state.pending_card_op.is_none() {
            return Ok(());
        }
        let Some(path) = self.cards_path.as_deref() else {
            self.state.pending_card_op = None;
            return Err("card data: cannot resolve ~/.hp41/cards (no $HOME)".to_string());
        };
        hp41_app::drain_pending_card_op(&mut self.state, path).map_err(|error| error.to_string())
    }

    fn apply_key(&mut self, key: &str) -> Result<(), String> {
        hp41_app::dispatch_request_key(&mut self.state, key)
    }

    fn press(&mut self, key: &str) {
        self.last_error = None;
        self.last_result = None;
        let result = self.apply_key(key).and_then(|_| self.drain_card_op());
        if let Err(message) = result {
            self.last_error = Some(message);
        }
        let _ = self.save();
    }

    fn request(&mut self, request: hp41_app::AppRequest) {
        let outcome = hp41_app::execute_request(
            &mut self.state,
            request,
            &self.state_path,
            self.cards_path.as_deref(),
        );
        self.last_error = outcome.error;
        self.last_result = outcome.result;
    }

    fn view(&mut self) -> hp41_app::AppResponse {
        hp41_app::AppResponse::drain(
            &mut self.state,
            self.last_error.clone(),
            self.last_result.take(),
        )
    }
}

fn c_string(value: String) -> *mut c_char {
    CString::new(value.replace('\0', " ")).map_or(std::ptr::null_mut(), CString::into_raw)
}

fn error_json(message: &str) -> *mut c_char {
    c_string(serde_json::json!({ "status": "error", "error": message }).to_string())
}

fn serialize_view(calculator: &mut Calculator) -> *mut c_char {
    match serde_json::to_string(&calculator.view()) {
        Ok(json) => c_string(json),
        Err(error) => error_json(&format!("response serialization failed: {error}")),
    }
}

fn catch_ffi<T>(operation: impl FnOnce() -> T, fallback: impl FnOnce() -> T) -> T {
    match catch_unwind(AssertUnwindSafe(operation)) {
        Ok(value) => value,
        Err(_) => fallback(),
    }
}

unsafe fn required_string<'a>(value: *const c_char) -> Option<&'a str> {
    if value.is_null() {
        return None;
    }
    CStr::from_ptr(value).to_str().ok()
}

#[no_mangle]
/// Creates a calculator using the state file at `state_path`.
///
/// # Safety
/// `state_path` must point to a valid, NUL-terminated UTF-8 string for the
/// duration of this call. The returned pointer must be released with
/// [`hp41_destroy`].
pub unsafe extern "C" fn hp41_create(state_path: *const c_char) -> *mut Calculator {
    catch_ffi(
        || {
            let Some(path) = required_string(state_path) else {
                return std::ptr::null_mut();
            };
            Box::into_raw(Box::new(Calculator::load(Path::new(path).to_path_buf())))
        },
        std::ptr::null_mut,
    )
}

#[no_mangle]
/// Releases a calculator created by [`hp41_create`].
///
/// # Safety
/// `calculator` must be null or a live pointer returned by [`hp41_create`], and
/// it must not be released more than once.
pub unsafe extern "C" fn hp41_destroy(calculator: *mut Calculator) {
    if !calculator.is_null() {
        drop(Box::from_raw(calculator));
    }
}

#[no_mangle]
/// Clones the calculator's atomic cancellation signal into an independent handle.
/// Create this before starting background execution and release it with
/// [`hp41_cancellation_handle_destroy`].
pub unsafe extern "C" fn hp41_cancellation_handle(
    calculator: *const Calculator,
) -> *mut CancellationHandle {
    let Some(calculator) = calculator.as_ref() else {
        return std::ptr::null_mut();
    };
    Box::into_raw(Box::new(CancellationHandle {
        signal: Arc::clone(&calculator.state.cancel_requested),
    }))
}

#[no_mangle]
/// Sets the shared cancellation flag. This function is safe to call from a
/// different thread while a calculator request is executing.
pub unsafe extern "C" fn hp41_cancel(handle: *const CancellationHandle) {
    if let Some(handle) = handle.as_ref() {
        handle.signal.store(true, Ordering::Relaxed);
    }
}

#[no_mangle]
pub unsafe extern "C" fn hp41_cancellation_handle_destroy(handle: *mut CancellationHandle) {
    if !handle.is_null() {
        drop(Box::from_raw(handle));
    }
}

#[no_mangle]
/// Sends one UTF-8 key identifier to the calculator.
///
/// # Safety
/// `calculator` must be a live pointer returned by [`hp41_create`]. `key` must
/// point to a valid, NUL-terminated UTF-8 string for the duration of this call.
pub unsafe extern "C" fn hp41_press(calculator: *mut Calculator, key: *const c_char) {
    let Some(calculator) = calculator.as_mut() else {
        return;
    };
    let result = catch_unwind(AssertUnwindSafe(|| {
        let Some(key) = required_string(key) else {
            calculator.last_error = Some("invalid key: expected non-null UTF-8".to_string());
            return;
        };
        calculator.press(key);
    }));
    if result.is_err() {
        calculator.last_error = Some("internal panic at FFI boundary".to_string());
    }
}

#[no_mangle]
/// Returns an allocated JSON snapshot of the calculator state.
///
/// # Safety
/// `calculator` must be a live pointer returned by [`hp41_create`]. The returned
/// string must be released with [`hp41_string_free`].
pub unsafe extern "C" fn hp41_state_json(calculator: *mut Calculator) -> *mut c_char {
    let Some(calculator) = calculator.as_mut() else {
        return std::ptr::null_mut();
    };
    catch_ffi(
        || serialize_view(calculator),
        || error_json("internal panic at FFI boundary"),
    )
}

#[no_mangle]
/// Executes a typed application request and returns the resulting full state JSON.
///
/// # Safety
/// `calculator` must be a live pointer returned by [`hp41_create`]. `request_json`
/// must point to valid, NUL-terminated UTF-8 JSON for the duration of this call.
/// The returned string must be released with [`hp41_string_free`].
pub unsafe extern "C" fn hp41_request_json(
    calculator: *mut Calculator,
    request_json: *const c_char,
) -> *mut c_char {
    let Some(calculator) = calculator.as_mut() else {
        return std::ptr::null_mut();
    };
    catch_ffi(
        || {
            let Some(request_json) = required_string(request_json) else {
                calculator.last_error =
                    Some("invalid request: expected non-null UTF-8 JSON".to_string());
                return serialize_view(calculator);
            };
            match serde_json::from_str::<hp41_app::AppRequest>(request_json) {
                Ok(request) => calculator.request(request),
                Err(error) => calculator.last_error = Some(format!("invalid request: {error}")),
            }
            serialize_view(calculator)
        },
        || error_json("internal panic at FFI boundary"),
    )
}

#[no_mangle]
/// Releases a string returned by [`hp41_state_json`].
///
/// # Safety
/// `value` must be null or a live pointer returned by [`hp41_state_json`], and
/// it must not be released more than once.
pub unsafe extern "C" fn hp41_string_free(value: *mut c_char) {
    if !value.is_null() {
        drop(CString::from_raw(value));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hp41_app::{AppRequest as BridgeRequest, AppResult as BridgeResult};
    use serde_json::Value;

    const GOLDEN_FLOWS: &str =
        include_str!("../../Tests/HP41GUITests/Fixtures/bridge-golden-flows.json");

    fn assert_json_pointer(actual: &Value, pointer: &str, expected: &Value, case_name: &str) {
        assert_eq!(
            actual.pointer(pointer),
            Some(expected),
            "golden flow {case_name:?} mismatch at {pointer}"
        );
    }

    unsafe fn take_ffi_json(pointer: *mut c_char) -> Value {
        assert!(!pointer.is_null(), "FFI must return a JSON string");
        let text = CStr::from_ptr(pointer)
            .to_str()
            .expect("FFI response must be UTF-8")
            .to_string();
        hp41_string_free(pointer);
        serde_json::from_str(&text).expect("FFI response must be JSON")
    }

    unsafe fn ffi_request(calculator: *mut Calculator, request: &Value) -> Value {
        let request = CString::new(request.to_string()).expect("request must not contain NUL");
        take_ffi_json(hp41_request_json(calculator, request.as_ptr()))
    }

    #[test]
    fn exported_abi_reports_explicit_status_and_validates_text_inputs() {
        let path =
            CString::new(format!("/tmp/hp41-ffi-status-{}.json", std::process::id())).unwrap();
        let calculator = unsafe { hp41_create(path.as_ptr()) };
        assert!(!calculator.is_null());

        let state = unsafe { take_ffi_json(hp41_state_json(calculator)) };
        assert_eq!(state["status"], "ok");
        assert_eq!(state["error"], Value::Null);

        let malformed = CString::new("{").unwrap();
        let response = unsafe { take_ffi_json(hp41_request_json(calculator, malformed.as_ptr())) };
        assert_eq!(response["status"], "error");
        assert!(response["error"]
            .as_str()
            .unwrap()
            .starts_with("invalid request:"));

        let response = unsafe { take_ffi_json(hp41_request_json(calculator, std::ptr::null())) };
        assert_eq!(response["status"], "error");
        assert_eq!(
            response["error"],
            "invalid request: expected non-null UTF-8 JSON"
        );

        let invalid_utf8 = [0xff_u8, 0];
        let response =
            unsafe { take_ffi_json(hp41_request_json(calculator, invalid_utf8.as_ptr().cast())) };
        assert_eq!(response["status"], "error");

        let response = unsafe {
            ffi_request(
                calculator,
                &serde_json::json!({ "command": "dispatch", "key_id": "1" }),
            )
        };
        assert_eq!(response["status"], "ok");
        assert_eq!(response["error"], Value::Null);

        let response = unsafe {
            ffi_request(
                calculator,
                &serde_json::json!({ "command": "dispatch", "key_id": "not_a_real_key" }),
            )
        };
        assert_eq!(response["status"], "error");
        assert_eq!(response["error"], "unknown key: not_a_real_key");

        unsafe {
            hp41_press(calculator, std::ptr::null());
        }
        let response = unsafe { take_ffi_json(hp41_state_json(calculator)) };
        assert_eq!(response["status"], "error");
        assert_eq!(response["error"], "invalid key: expected non-null UTF-8");

        assert!(unsafe { hp41_create(std::ptr::null()) }.is_null());
        let invalid_path = [0xff_u8, 0];
        assert!(unsafe { hp41_create(invalid_path.as_ptr().cast()) }.is_null());
        assert!(unsafe { hp41_state_json(std::ptr::null_mut()) }.is_null());
        assert!(unsafe { hp41_request_json(std::ptr::null_mut(), malformed.as_ptr()) }.is_null());
        unsafe { hp41_destroy(calculator) };
    }

    #[test]
    fn exported_abi_loads_retained_state_fixture_compatibly() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../src-tauri/tests/fixtures/v40-autosave.json");
        let path = CString::new(path.to_string_lossy().as_bytes()).unwrap();
        let calculator = unsafe { hp41_create(path.as_ptr()) };
        assert!(!calculator.is_null());
        let response = unsafe { take_ffi_json(hp41_state_json(calculator)) };
        assert_eq!(response["status"], "ok");
        for field in [
            "display_str",
            "x_str",
            "annunciators",
            "program_steps",
            "pending_yield",
        ] {
            assert!(
                response.get(field).is_some(),
                "missing compatible state field {field}"
            );
        }
        unsafe { hp41_destroy(calculator) };
    }

    #[test]
    fn every_app_request_variant_round_trips_through_exported_abi() {
        let root = std::env::temp_dir().join(format!("hp41-ffi-variants-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let state_path =
            CString::new(root.join("state.json").to_string_lossy().as_bytes()).unwrap();
        let calculator = unsafe { hp41_create(state_path.as_ptr()) };
        assert!(!calculator.is_null());

        let missing_raw = root.join("missing.raw").to_string_lossy().to_string();
        let missing_data = root.join("missing.card.json").to_string_lossy().to_string();
        let export_raw = root.join("export.raw").to_string_lossy().to_string();
        let export_data = root.join("export.card.json").to_string_lossy().to_string();
        let requests = vec![
            serde_json::json!({ "command": "get_state" }),
            serde_json::json!({ "command": "dispatch", "key_id": "1" }),
            serde_json::json!({ "command": "sst_step" }),
            serde_json::json!({ "command": "bst_step" }),
            serde_json::json!({ "command": "run_stop" }),
            serde_json::json!({ "command": "run_program", "label": "MISSING" }),
            serde_json::json!({ "command": "resume_program" }),
            serde_json::json!({ "command": "resume_program_with_key", "keycode": 11 }),
            serde_json::json!({ "command": "submit_modal" }),
            serde_json::json!({ "command": "cancel_modal" }),
            serde_json::json!({ "command": "submit_modal_with_label", "label": "A" }),
            serde_json::json!({ "command": "tick_time" }),
            serde_json::json!({ "command": "save_state" }),
            serde_json::json!({ "command": "reset_soft" }),
            serde_json::json!({ "command": "reset_full" }),
            serde_json::json!({ "command": "inspect_raw", "path": missing_raw.clone() }),
            serde_json::json!({ "command": "import_raw", "path": missing_raw, "indices": [0] }),
            serde_json::json!({ "command": "export_raw", "path": export_raw }),
            serde_json::json!({ "command": "import_data", "path": missing_data }),
            serde_json::json!({ "command": "export_data", "path": export_data }),
            serde_json::json!({ "command": "request_cancel" }),
        ];

        for request in requests {
            let response = unsafe { ffi_request(calculator, &request) };
            let status = response["status"].as_str();
            assert!(
                matches!(status, Some("ok" | "error")),
                "request {request} returned no valid status: {response}"
            );
            assert_eq!(status == Some("error"), !response["error"].is_null());
        }

        unsafe { hp41_destroy(calculator) };
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn exported_abi_supports_repeated_create_cancel_destroy_lifecycles() {
        for index in 0..32 {
            let path = CString::new(format!(
                "/tmp/hp41-ffi-lifecycle-{}-{index}.json",
                std::process::id()
            ))
            .unwrap();
            let calculator = unsafe { hp41_create(path.as_ptr()) };
            assert!(!calculator.is_null());
            let cancellation = unsafe { hp41_cancellation_handle(calculator) };
            assert!(!cancellation.is_null());
            unsafe {
                hp41_cancel(cancellation);
                hp41_cancellation_handle_destroy(cancellation);
                hp41_destroy(calculator);
            }
        }
        unsafe {
            hp41_cancel(std::ptr::null());
            hp41_cancellation_handle_destroy(std::ptr::null_mut());
            hp41_destroy(std::ptr::null_mut());
            hp41_string_free(std::ptr::null_mut());
        }
    }

    #[test]
    fn panic_guard_returns_fallback_without_unwinding() {
        assert_eq!(catch_ffi(|| panic!("synthetic FFI panic"), || 41), 41);
    }

    #[test]
    fn shared_golden_flows_replay_through_bridge() {
        let fixture: Value = serde_json::from_str(GOLDEN_FLOWS).expect("golden fixture must parse");
        assert_eq!(fixture["schema_version"], 1);
        let cases = fixture["cases"]
            .as_array()
            .expect("fixture cases must be an array");
        assert_eq!(
            cases.len(),
            11,
            "all planned representative categories must remain covered"
        );

        let root = std::env::temp_dir().join(format!("hp41-golden-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("golden temp directory must be creatable");

        for case in cases {
            let name = case["name"].as_str().expect("case name must be a string");
            let case_root = root.join(name);
            let cards_path = case_root.join("cards");
            fs::create_dir_all(&cards_path).expect("case cards directory must be creatable");
            let mut calculator = Calculator::load(case_root.join("state.json"));
            calculator.cards_path = Some(cards_path.clone());

            let requests = case["requests"]
                .as_array()
                .expect("requests must be an array");
            let mut final_response = Value::Null;
            for request_value in requests {
                let request: BridgeRequest = serde_json::from_value(request_value.clone())
                    .unwrap_or_else(|error| panic!("invalid request in {name:?}: {error}"));
                calculator.request(request);
                final_response = serde_json::to_value(calculator.view())
                    .expect("bridge response must serialize");
            }

            for assertion in case["assertions"]
                .as_array()
                .expect("assertions must be an array")
            {
                let pointer = assertion["pointer"]
                    .as_str()
                    .expect("pointer must be a string");
                assert_json_pointer(&final_response, pointer, &assertion["equals"], name);
            }
            if let Some(artifact) = case.get("artifact").and_then(Value::as_str) {
                assert!(
                    cards_path.join(artifact).is_file(),
                    "golden flow {name:?} did not create {artifact}"
                );
            }
        }

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn rpn_smoke() {
        let path = PathBuf::from("/tmp/hp41-bridge-rpn-smoke.json");
        let _ = fs::remove_file(&path);
        let mut calculator = Calculator::load(path.clone());
        for key in ["2", "enter", "3", "plus"] {
            calculator.press(key);
        }
        assert_eq!(calculator.view().state.x_str, "5.0000");
        let _ = fs::remove_file(path);
    }

    #[test]
    fn bridge_uses_shared_exhaustive_resolver() {
        let path = PathBuf::from("/tmp/hp41-bridge-shared-resolver.json");
        let _ = fs::remove_file(&path);
        let mut calculator = Calculator::load(path.clone());

        // `abs` and parameterized `fix_N` were absent from the bridge's former
        // hand-written resolver but are part of the Tauri resolver contract.
        for key in ["2", "chs", "abs", "fix_2"] {
            calculator.press(key);
        }

        assert_eq!(calculator.view().state.x_str, "2.00");
        assert_eq!(calculator.view().error, None);
        let _ = fs::remove_file(path);
    }

    #[test]
    fn typed_dispatch_request_matches_compatibility_press_and_preserves_errors() {
        let path = PathBuf::from("/tmp/hp41-bridge-typed-dispatch.json");
        let _ = fs::remove_file(&path);
        let mut calculator = Calculator::load(path.clone());

        for key_id in ["2", "enter", "3", "plus"] {
            calculator.request(BridgeRequest::Dispatch {
                key_id: key_id.to_string(),
            });
        }
        let result = calculator.view();
        assert_eq!(result.state.x_str, "5.0000");
        assert_eq!(result.error, None);

        calculator.request(BridgeRequest::Dispatch {
            key_id: "not_a_real_key".to_string(),
        });
        assert_eq!(
            calculator.view().error.as_deref(),
            Some("unknown key: not_a_real_key")
        );

        calculator.request(BridgeRequest::Dispatch {
            key_id: "1".to_string(),
        });
        assert_eq!(calculator.view().error, None);
        let _ = fs::remove_file(path);
    }

    #[test]
    fn dispatch_request_json_requires_a_string_key_id() {
        assert!(matches!(
            serde_json::from_str::<BridgeRequest>(r#"{"command":"dispatch","key_id":"abs"}"#),
            Ok(BridgeRequest::Dispatch { key_id }) if key_id == "abs"
        ));
        assert!(
            serde_json::from_str::<BridgeRequest>(r#"{"command":"dispatch","key_id":7}"#).is_err()
        );
        assert!(serde_json::from_str::<BridgeRequest>(r#"{"command":"dispatch"}"#).is_err());
    }

    #[test]
    fn every_remaining_inventory_direct_id_is_accepted_by_typed_dispatch() {
        const IDS: &[&str] = &[
            "abs",
            "alpha_clear",
            "aoff",
            "aon",
            "arot",
            "asn",
            "atox",
            "aview",
            "beep",
            "catalog",
            "cf_prompt",
            "cl_sigma_stat",
            "cla",
            "cld",
            "clreg",
            "clst",
            "corr",
            "eex_chs",
            "eng_prompt",
            "fact",
            "fix_prompt",
            "frc",
            "fs_prompt",
            "getkey",
            "gto_prompt",
            "h_to_hms",
            "hms_add",
            "hms_sub",
            "hms_to_h",
            "ins",
            "int",
            "isg_prompt",
            "lbl_prompt",
            "lr",
            "mean",
            "mod_op",
            "null",
            "pack",
            "posa",
            "pra",
            "prompt",
            "prstk",
            "prx",
            "pse",
            "r_up",
            "rcl_m",
            "rcl_n",
            "rcl_o",
            "rcl_prompt",
            "rnd",
            "sci_prompt",
            "sdev",
            "set_deg",
            "set_grad",
            "set_rad",
            "sf_prompt",
            "sign",
            "sto_m",
            "sto_n",
            "sto_o",
            "sto_prompt",
            "stop",
            "sw_exit",
            "tone",
            "view",
            "x_eq_0",
            "x_eq_0_prompt",
            "x_eq_y",
            "x_eq_y_prompt",
            "x_gt_y",
            "x_gt_y_prompt",
            "x_le_y",
            "x_le_y_prompt",
            "xeq_prompt",
            "xtoa",
            "xy_swap",
            "yhat",
        ];
        assert_eq!(IDS.len(), 77);
        let path = PathBuf::from("/tmp/hp41-bridge-direct-inventory.json");
        let _ = fs::remove_file(&path);
        let mut calculator = Calculator::load(path.clone());

        for key_id in IDS {
            calculator.state = CalcState::new();
            if let Err(error) = calculator.apply_key(key_id) {
                assert!(
                    !error.starts_with("unknown key:"),
                    "typed dispatch does not accept inventory ID {key_id}: {error}"
                );
            }
        }
        let _ = fs::remove_file(path);
    }

    #[test]
    fn remaining_fc_parameter_families_use_shared_resolver() {
        assert!(hp41_app::resolve("fc_05").is_ok());
        assert!(hp41_app::resolve("fc_ind_05").is_ok());
    }

    #[test]
    fn independent_cancellation_handle_shares_only_atomic_signal() {
        let path = PathBuf::from("/tmp/hp41-bridge-cancel-handle.json");
        let calculator = Calculator::load(path.clone());
        let handle = CancellationHandle {
            signal: Arc::clone(&calculator.state.cancel_requested),
        };
        let worker_signal = Arc::clone(&handle.signal);
        let worker = std::thread::spawn(move || worker_signal.store(true, Ordering::Relaxed));
        worker.join().expect("cancel worker must finish");
        assert!(calculator.state.cancel_requested.load(Ordering::Relaxed));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn bridge_projects_and_drains_complete_shared_view() {
        let path = PathBuf::from("/tmp/hp41-bridge-shared-view.json");
        let _ = fs::remove_file(&path);
        let mut calculator = Calculator::load(path.clone());
        calculator.state.print_buffer.push("PRINT".to_string());
        calculator.state.event_buffer.push("BEEP".to_string());
        calculator.state.flags |= 1 << 5;

        let first = calculator.view();
        assert_eq!(first.state.print_lines, ["PRINT"]);
        assert_eq!(first.state.event_buffer, ["BEEP"]);
        assert_eq!(first.state.flags, [5]);
        assert_eq!(first.state.program_steps, ["000 END"]);

        let second = calculator.view();
        assert!(second.state.print_lines.is_empty());
        assert!(second.state.event_buffer.is_empty());
        let _ = fs::remove_file(path);
    }

    #[test]
    fn request_boundary_runs_and_resumes_getkey_program() {
        let path = PathBuf::from("/tmp/hp41-bridge-request-getkey.json");
        let _ = fs::remove_file(&path);
        let mut calculator = Calculator::load(path.clone());
        calculator.state.program = vec![
            hp41_core::ops::Op::Lbl("A".to_string()),
            hp41_core::ops::Op::GetKey,
            hp41_core::ops::Op::PushNum(hp41_core::HpNum::from(1)),
            hp41_core::ops::Op::Add,
        ];

        calculator.request(BridgeRequest::RunProgram {
            label: "A".to_string(),
        });
        assert_eq!(
            calculator
                .view()
                .state
                .pending_yield
                .as_ref()
                .map(|pending| pending.kind.as_str()),
            Some("wait_for_key")
        );

        calculator.request(BridgeRequest::ResumeProgramWithKey { keycode: 11 });
        let finished = calculator.view();
        assert_eq!(finished.state.x_str, "12.0000");
        assert!(finished.state.pending_yield.is_none());
        assert_eq!(finished.error, None);
        let _ = fs::remove_file(path);
    }

    #[test]
    fn file_requests_export_inspect_and_import_raw() {
        let state_path = PathBuf::from("/tmp/hp41-bridge-file-state.json");
        let raw_path = PathBuf::from("/tmp/hp41-bridge-file-program.raw");
        let _ = fs::remove_file(&state_path);
        let _ = fs::remove_file(&raw_path);
        let mut calculator = Calculator::load(state_path.clone());
        calculator.state.program = vec![
            hp41_core::ops::Op::Lbl("FILE".to_string()),
            hp41_core::ops::Op::Add,
        ];

        calculator.request(BridgeRequest::ExportRaw {
            path: raw_path.to_string_lossy().into_owned(),
        });
        assert!(raw_path.is_file());
        calculator.request(BridgeRequest::InspectRaw {
            path: raw_path.to_string_lossy().into_owned(),
        });
        let inspection = calculator.view().result;
        assert!(matches!(
            inspection,
            Some(BridgeResult::RawPrograms { programs }) if programs.len() == 1
                && programs[0].label.starts_with("FILE (")
        ));

        calculator.state.program.clear();
        calculator.request(BridgeRequest::ImportRaw {
            path: raw_path.to_string_lossy().into_owned(),
            indices: vec![0],
        });
        assert!(matches!(
            calculator.state.program.first(),
            Some(hp41_core::ops::Op::Lbl(label)) if label == "FILE"
        ));
        assert_eq!(calculator.last_error, None);
        let _ = fs::remove_file(state_path);
        let _ = fs::remove_file(raw_path);
    }

    #[test]
    fn dispatched_card_reader_operations_round_trip_through_shared_directory() {
        let state_path = PathBuf::from("/tmp/hp41-bridge-card-state.json");
        let cards_path = PathBuf::from("/tmp/hp41-bridge-cards");
        let _ = fs::remove_file(&state_path);
        let _ = fs::remove_dir_all(&cards_path);
        let mut calculator = Calculator::load(state_path.clone());
        calculator.cards_path = Some(cards_path.clone());
        calculator.state.program = vec![hp41_core::ops::Op::Add];
        calculator.state.alpha_reg = "NATIVE".to_string();

        calculator.press("xeq_WPRGM");
        assert!(cards_path.join("NATIVE.raw").is_file());
        calculator.state.program.clear();
        calculator.press("xeq_RDPRGM");
        assert_eq!(calculator.state.program, [hp41_core::ops::Op::Add]);

        calculator.state.regs[7] = hp41_core::HpValue::Numeric(hp41_core::HpNum::from(41));
        calculator.state.alpha_reg = "REGS".to_string();
        calculator.press("xeq_WDTA");
        assert!(cards_path.join("REGS.card.json").is_file());
        calculator.state.regs[7] = hp41_core::HpValue::default();
        calculator.press("xeq_RDTA");
        assert_eq!(
            calculator.state.regs[7],
            hp41_core::HpValue::Numeric(hp41_core::HpNum::from(41))
        );
        assert!(calculator.state.pending_card_op.is_none());
        assert_eq!(calculator.last_error, None);
        let _ = fs::remove_file(state_path);
        let _ = fs::remove_dir_all(cards_path);
    }
}
