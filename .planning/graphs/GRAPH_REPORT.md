# Graph Report - hp41-calculator-emulator  (2026-05-11)

## Corpus Check
- 69 files · ~76,443 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 1105 nodes · 2174 edges · 54 communities (49 shown, 5 thin omitted)
- Extraction: 74% EXTRACTED · 26% INFERRED · 0% AMBIGUOUS · INFERRED: 561 edges (avg confidence: 0.8)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `cc11865b`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- [[_COMMUNITY_Community 0|Community 0]]
- [[_COMMUNITY_Community 1|Community 1]]
- [[_COMMUNITY_Community 2|Community 2]]
- [[_COMMUNITY_Community 3|Community 3]]
- [[_COMMUNITY_Community 4|Community 4]]
- [[_COMMUNITY_Community 5|Community 5]]
- [[_COMMUNITY_Community 6|Community 6]]
- [[_COMMUNITY_Community 7|Community 7]]
- [[_COMMUNITY_Community 8|Community 8]]
- [[_COMMUNITY_Community 9|Community 9]]
- [[_COMMUNITY_Community 10|Community 10]]
- [[_COMMUNITY_Community 11|Community 11]]
- [[_COMMUNITY_Community 12|Community 12]]
- [[_COMMUNITY_Community 13|Community 13]]
- [[_COMMUNITY_Community 14|Community 14]]
- [[_COMMUNITY_Community 15|Community 15]]
- [[_COMMUNITY_Community 16|Community 16]]
- [[_COMMUNITY_Community 17|Community 17]]
- [[_COMMUNITY_Community 18|Community 18]]
- [[_COMMUNITY_Community 19|Community 19]]
- [[_COMMUNITY_Community 20|Community 20]]
- [[_COMMUNITY_Community 21|Community 21]]
- [[_COMMUNITY_Community 22|Community 22]]
- [[_COMMUNITY_Community 23|Community 23]]
- [[_COMMUNITY_Community 24|Community 24]]
- [[_COMMUNITY_Community 25|Community 25]]
- [[_COMMUNITY_Community 26|Community 26]]
- [[_COMMUNITY_Community 27|Community 27]]
- [[_COMMUNITY_Community 28|Community 28]]
- [[_COMMUNITY_Community 29|Community 29]]
- [[_COMMUNITY_Community 30|Community 30]]
- [[_COMMUNITY_Community 31|Community 31]]
- [[_COMMUNITY_Community 32|Community 32]]
- [[_COMMUNITY_Community 33|Community 33]]
- [[_COMMUNITY_Community 34|Community 34]]
- [[_COMMUNITY_Community 35|Community 35]]
- [[_COMMUNITY_Community 36|Community 36]]
- [[_COMMUNITY_Community 37|Community 37]]
- [[_COMMUNITY_Community 38|Community 38]]
- [[_COMMUNITY_Community 39|Community 39]]
- [[_COMMUNITY_Community 40|Community 40]]
- [[_COMMUNITY_Community 41|Community 41]]
- [[_COMMUNITY_Community 42|Community 42]]
- [[_COMMUNITY_Community 45|Community 45]]
- [[_COMMUNITY_Community 47|Community 47]]
- [[_COMMUNITY_Community 48|Community 48]]

## God Nodes (most connected - your core abstractions)
1. `dispatch()` - 297 edges
2. `execute_op()` - 64 edges
3. `run_program()` - 49 edges
4. `apply_lift_effect()` - 47 edges
5. `make_app()` - 42 edges
6. `HpNum` - 36 edges
7. `unary_result()` - 24 edges
8. `state_with_program()` - 23 edges
9. `push()` - 20 edges
10. `push_dec()` - 20 edges

## Surprising Connections (you probably didn't know these)
- `test_user_mode_dispatch()` --calls--> `dispatch()`  [INFERRED]
  hp41-cli/src/keys.rs → hp41-core/src/ops/mod.rs
- `test_q_dispatches_sin()` --calls--> `dispatch()`  [INFERRED]
  hp41-cli/src/keys.rs → hp41-core/src/ops/mod.rs
- `test_g_dispatches_clreg()` --calls--> `dispatch()`  [INFERRED]
  hp41-cli/src/keys.rs → hp41-core/src/ops/mod.rs
- `test_integration_1_eex_chs_2_enter_gives_0_01()` --calls--> `format_hpnum()`  [INFERRED]
  hp41-cli/src/app.rs → hp41-core/src/format.rs
- `test_eex_trailing_e_then_enter_pushes_mantissa()` --calls--> `dispatch()`  [INFERRED]
  hp41-cli/src/app.rs → hp41-core/src/ops/mod.rs

## Communities (54 total, 5 thin omitted)

### Community 0 - "Community 0"
Cohesion: 0.05
Nodes (73): build_counter(), evaluate_test(), find_in_program(), find_label_in_state(), op_dse(), op_gto(), op_isg(), op_lbl() (+65 more)

### Community 1 - "Community 1"
Cohesion: 0.03
Nodes (18): apply_lift_effect_disable_sets_false(), apply_lift_effect_enable_sets_true(), apply_lift_effect_neutral_leaves_unchanged(), dispatch_add_works(), dispatch_div_by_zero_propagates_error(), dispatch_push_num_enters_value(), enter_number_lifts_when_lift_enabled(), enter_number_overwrites_x_when_lift_disabled() (+10 more)

### Community 2 - "Community 2"
Cohesion: 0.11
Nodes (51): App(), key(), make_app(), make_key(), PendingInput, press(), test_autosave_timer_logic(), test_autosave_timer_no_premature_save() (+43 more)

### Community 3 - "Community 3"
Cohesion: 0.07
Nodes (36): bst_step(), dispatch_op(), get_state(), handle_bst(), handle_get_state(), handle_op(), handle_sst(), sst_step() (+28 more)

### Community 4 - "Community 4"
Cohesion: 0.04
Nodes (46): 10.1 Dependencies, 10.2 Approvals, 10. Timeline and Milestones, 1. Executive Summary, 2.1 Current State, 2.2 Problem Description, 2.3 Impact, 2.4 Evidence (+38 more)

### Community 5 - "Community 5"
Cohesion: 0.08
Nodes (34): bench_dispatch_1000_ops(), bench_dispatch_mixed(), bench_dispatch_single_add(), dispatch(), test_alpha_24_char_limit_enforced(), test_alpha_append_builds_string(), test_alpha_clear_empties_register(), test_alpha_ops_are_neutral_lift() (+26 more)

### Community 6 - "Community 6"
Cohesion: 0.05
Nodes (39): Alpha ↔ Numbers, Alpha Register, Branching, Building a String, code:block1 (Register  Content        Role), code:block10 (current.FFFstep), code:block11 (1.00500  →  STO 00      (counter from 1 to 5, step 1)), code:block12 (5.00100  →  STO 01      (counter from 5 to 1, step 1)) (+31 more)

### Community 7 - "Community 7"
Cohesion: 0.1
Nodes (26): assemble_sci(), compute_sci_exp(), decimal_pow10(), ensure_decimal_point(), floor_to_multiple_of_3(), format_alpha(), format_eng(), format_fix() (+18 more)

### Community 8 - "Community 8"
Cohesion: 0.09
Nodes (35): accumulate_two_points(), make_state_with_values(), push_dec(), test_add_enables_lift(), test_alpha_ops_neutral_lift_in_lift_tests(), test_chs_neutral_lift_false(), test_chs_neutral_lift_true(), test_clreg_neutral_lift() (+27 more)

### Community 10 - "Community 10"
Cohesion: 0.17
Nodes (27): op_pra(), op_prstk(), op_prx(), execute_op(), op_getkey(), op_rcl(), op_rcl_m(), op_rcl_n() (+19 more)

### Community 11 - "Community 11"
Cohesion: 0.11
Nodes (24): synthetic_byte_to_op(), push(), test_getkey_in_program(), test_getkey_lifts_stack(), test_getkey_pushes_last_key_code(), test_getkey_zero_when_no_key_pressed(), test_hidden_reg_in_program(), test_hidden_regs_are_independent() (+16 more)

### Community 12 - "Community 12"
Cohesion: 0.12
Nodes (28): f64_from_radians(), op_acos(), op_asin(), op_atan(), op_cos(), op_exp(), op_int(), op_ln() (+20 more)

### Community 13 - "Community 13"
Cohesion: 0.1
Nodes (4): test_evaluate_test_relational_variants(), HpNum, test_hpnum_serde_decimal_precision(), test_hpnum_serde_is_string()

### Community 14 - "Community 14"
Cohesion: 0.08
Nodes (25): Adding a New Operation, CalcState — Single Source of Truth, code:block1 (hp41-calculator-emulator/), code:rust (pub fn dispatch(state: &mut CalcState, op: Op) -> Result<Lif), code:rust (pub struct Stack {), code:rust (pub enum LiftEffect { Enable, Disable, Neutral }), code:rust (pub enum StoArithKind { Add, Sub, Mul, Div }), code:rust (pub struct StateFile {) (+17 more)

### Community 15 - "Community 15"
Cohesion: 0.11
Nodes (10): sample_programs(), SampleProgram, test_all_programs_non_empty(), test_all_programs_start_with_lbl_a(), test_fibonacci_runs_without_panic(), test_gcd_correctness(), test_prime_test_correctness(), test_program_names_unique() (+2 more)

### Community 16 - "Community 16"
Cohesion: 0.21
Nodes (23): push(), push_dec(), set_deg(), set_grad(), set_rad(), test_acos_half_is_60_deg(), test_acos_out_of_domain_returns_error(), test_angle_mode_stored_after_set_deg() (+15 more)

### Community 17 - "Community 17"
Cohesion: 0.09
Nodes (21): Alpha Register & String Operations, Angle Modes, Arithmetic, Conditional Tests (skip next step if true), Conversions, Flags, HP-41CX: Time Module (Built-in), Hyperbolics (via X-Functions module / HP-41CX) (+13 more)

### Community 18 - "Community 18"
Cohesion: 0.19
Nodes (20): push(), push_dec(), test_exp_of_0_is_1(), test_ln_2_accuracy_10_digits(), test_ln_of_1_is_0(), test_ln_of_negative_returns_domain(), test_ln_of_zero_returns_domain(), test_log_of_100_is_2() (+12 more)

### Community 19 - "Community 19"
Cohesion: 0.17
Nodes (19): push(), test_add_consumes_y(), test_chs_negates_x(), test_chs_neutral_lift_when_disabled(), test_chs_neutral_lift_when_enabled(), test_clx_disables_lift(), test_clx_zeros_x(), test_div_by_zero_returns_error() (+11 more)

### Community 20 - "Community 20"
Cohesion: 0.19
Nodes (17): add_point(), push(), test_cl_sigma_stat_lift_is_neutral(), test_cl_sigma_stat_zeros_r01_to_r06(), test_corr_denominator_zero_returns_error(), test_corr_perfect_positive_correlation(), test_corr_singular_returns_error(), test_lr_slope_in_y_intercept_in_x() (+9 more)

### Community 21 - "Community 21"
Cohesion: 0.14
Nodes (19): push_val(), test_pra_empty_alpha_is_24_spaces(), test_pra_in_program(), test_pra_output_is_24_chars(), test_pra_output_is_left_aligned(), test_pra_pushes_one_line_to_buffer(), test_pra_truncates_long_alpha_to_24_chars(), test_prstk_all_lines_are_24_chars() (+11 more)

### Community 22 - "Community 22"
Cohesion: 0.19
Nodes (19): push_dec(), test_h_to_hms_canonical_1_5125(), test_h_to_hms_negative_value(), test_h_to_hms_round_trip(), test_hms_add_invalid_x_operand_returns_invalid_input(), test_hms_add_invalid_y_operand_returns_invalid_input(), test_hms_add_no_carry(), test_hms_add_with_carry() (+11 more)

### Community 23 - "Community 23"
Cohesion: 0.17
Nodes (17): op_add(), op_div(), op_mul(), op_sub(), op_ypow(), binary_result(), binary_result_captures_lastx_before_overwrite(), binary_result_enables_lift() (+9 more)

### Community 24 - "Community 24"
Cohesion: 0.21
Nodes (15): flush_entry_buf(), make_state_with_entry(), Op, StackReg, StoArithKind, test_flush_entry_buf_negative_exponent(), test_flush_implicit_one_with_trailing_e_commits_one(), test_flush_invalid_returns_err() (+7 more)

### Community 25 - "Community 25"
Cohesion: 0.19
Nodes (11): arithmetic_keys(), backspace_maps_to_op_clx(), enter_maps_to_op_enter(), f_keys_handled_in_app_return_none(), g_maps_to_clreg(), inverse_trig_lowercase(), make_app(), q_maps_to_sin() (+3 more)

### Community 27 - "Community 27"
Cohesion: 0.2
Nodes (14): push(), test_clreg_zeros_all_registers(), test_rcl_enables_lift(), test_rcl_pushes_to_stack(), test_sto_add_updates_register(), test_sto_arith_is_neutral_lift(), test_sto_div_updates_register(), test_sto_does_not_change_x() (+6 more)

### Community 28 - "Community 28"
Cohesion: 0.12
Nodes (15): code:block1 (┌─────────────────────────────────────┐), code:bash (# Prerequisites: Rust stable (MSRV 1.88), just), code:bash (just run                # build + launch the TUI), code:bash (# Additional prerequisites: Node.js + npm; see hp41-gui/READ), code:block5 (hp41-core/                — UI-agnostic library (calculator ), Contributing, Documentation, Features (+7 more)

### Community 29 - "Community 29"
Cohesion: 0.39
Nodes (12): default_state_path(), load_state(), save_state(), StateFile, temp_path(), test_corrupt_json_returns_err(), test_is_running_reset_on_load(), test_loads_v1_format_state_file() (+4 more)

### Community 30 - "Community 30"
Cohesion: 0.13
Nodes (13): checked_asin_of_zero_is_zero(), checked_atan_of_zero_is_zero(), checked_cos_of_zero_is_one(), checked_exp10_of_zero_is_one(), checked_exp_of_zero_is_one(), checked_ln_of_zero_returns_domain(), checked_log10_of_zero_returns_domain(), checked_recip_of_zero_returns_divide_by_zero() (+5 more)

### Community 31 - "Community 31"
Cohesion: 0.14
Nodes (13): Branch Model, Code of Conduct, code:block1 (fork → feat/your-feature → PR → develop → (release) → main), code:bash (just ci          # lint → test → coverage ≥80%), code:bash (just lint        # cargo clippy -D warnings + rustfmt check), code:bash (cargo install just cargo-llvm-cov), code:block5 (✨ feat(core): add ASIN operation), Commit Messages (+5 more)

### Community 32 - "Community 32"
Cohesion: 0.31
Nodes (12): d(), make_state(), op_clreg(), op_sto_arith_stack(), op_sto_m(), op_sto_n(), op_sto_o(), sto_arith_stack_add_y() (+4 more)

### Community 33 - "Community 33"
Cohesion: 0.15
Nodes (12): code:block1 (/gsd-progress           — check current status), Core engine (v1.0), Git Workflow, GSD Workflow, HP-41 Calculator Emulator — Project Guide, Key Files, Quality Gates (maintained across v1.0 → v2.0), Settled Architecture Decisions (+4 more)

### Community 34 - "Community 34"
Cohesion: 0.15
Nodes (12): code:block1 (┌───┬───────────┐), HP-41 Overview, HP-41C (1979), HP-41CV (1980), HP-41CX (1983), Memory and Registers, Official Documentation, RPN: Reverse Polish Notation (+4 more)

### Community 35 - "Community 35"
Cohesion: 0.39
Nodes (11): build_hms(), decimal_to_hms_fields(), hms_fields_to_decimal(), hms_to_total_secs(), op_h_to_hms(), op_hms_add(), op_hms_sub(), op_hms_to_h() (+3 more)

### Community 36 - "Community 36"
Cohesion: 0.3
Nodes (10): AccuracyCase, add_point(), dec(), get_x(), new_deg_state(), new_grad_state(), new_rad_state(), passes_with_tol() (+2 more)

### Community 37 - "Community 37"
Cohesion: 0.2
Nodes (8): Annunciators, CalcStateView, getKeyGrad(), isCreamKey(), KEY_DEFS, Keyboard(), KeyboardProps, KeyDef

### Community 38 - "Community 38"
Cohesion: 0.2
Nodes (5): key_to_op(), keycode_to_hp41_code(), test_g_dispatches_clreg(), test_q_dispatches_sin(), test_user_mode_dispatch()

### Community 39 - "Community 39"
Cohesion: 0.31
Nodes (4): AngleMode, CalcState, DisplayMode, Stack

### Community 40 - "Community 40"
Cohesion: 0.22
Nodes (8): code:block1 (┌───────────────────────────────────────────────────────────), code:block2 (XEQ       → primary function), Entry Keys, HP-41C/CV Keyboard Layout, Key Labeling Convention in This Emulator, Key Layout, Mode Keys, Numeric Pad

### Community 41 - "Community 41"
Cohesion: 0.25
Nodes (7): Attribution, Contributor Covenant Code of Conduct, Enforcement, Enforcement Responsibilities, Our Pledge, Our Standards, Scope

### Community 42 - "Community 42"
Cohesion: 0.38
Nodes (6): op_alpha_append(), op_alpha_backspace(), op_alpha_clear(), op_alpha_toggle(), test_alpha_backspace_on_empty_is_noop(), test_alpha_backspace_removes_last_char()

## Knowledge Gaps
- **158 isolated node(s):** `Cli`, `PendingInput`, `SampleProgram`, `AccuracyCase`, `HpError` (+153 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **5 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `dispatch()` connect `Community 5` to `Community 0`, `Community 1`, `Community 2`, `Community 3`, `Community 8`, `Community 9`, `Community 10`, `Community 11`, `Community 12`, `Community 15`, `Community 16`, `Community 18`, `Community 19`, `Community 20`, `Community 21`, `Community 22`, `Community 23`, `Community 24`, `Community 27`, `Community 32`, `Community 35`, `Community 36`, `Community 38`, `Community 42`?**
  _High betweenness centrality (0.480) - this node is a cross-community bridge._
- **Why does `handle_op()` connect `Community 3` to `Community 5`?**
  _High betweenness centrality (0.064) - this node is a cross-community bridge._
- **Why does `HpNum` connect `Community 13` to `Community 0`, `Community 12`, `Community 30`, `Community 7`?**
  _High betweenness centrality (0.061) - this node is a cross-community bridge._
- **Are the 293 inferred relationships involving `dispatch()` (e.g. with `test_user_mode_dispatch()` and `test_q_dispatches_sin()`) actually correct?**
  _`dispatch()` has 293 INFERRED edges - model-reasoned connections that need verification._
- **Are the 62 inferred relationships involving `execute_op()` (e.g. with `op_add()` and `op_sub()`) actually correct?**
  _`execute_op()` has 62 INFERRED edges - model-reasoned connections that need verification._
- **Are the 27 inferred relationships involving `run_program()` (e.g. with `.handle_key()` and `.try_user_dispatch()`) actually correct?**
  _`run_program()` has 27 INFERRED edges - model-reasoned connections that need verification._
- **Are the 46 inferred relationships involving `apply_lift_effect()` (e.g. with `apply_lift_effect_enable_sets_true()` and `apply_lift_effect_disable_sets_false()`) actually correct?**
  _`apply_lift_effect()` has 46 INFERRED edges - model-reasoned connections that need verification._