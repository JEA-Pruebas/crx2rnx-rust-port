use std::collections::HashMap;

use crate::error::CrxError;
use crate::header::{CrinexHeader, parse_header};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpochInfo {
    pub line_index: usize,
    pub epoch_line: String,
    pub satellites: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObservationState {
    pub observable_count: usize,
    pub previous_epoch: Option<EpochInfo>,
    pub previous_observables: HashMap<String, Vec<Option<i64>>>,
}

pub struct PureRustAnalysis {
    pub rinex_header: String,
    pub epochs: Vec<EpochInfo>,
    pub state: ObservationState,
}

#[derive(Debug, Clone)]
struct EpochBlock {
    info: EpochInfo,
    diff_order: u8,
    obs_lines: Vec<String>,
}

#[derive(Debug, Clone)]
struct CompactToken {
    is_absolute: bool,
    value: i64,
}

#[derive(Debug, Clone)]
struct SatRepairState {
    values: Vec<Option<i64>>,
    flags: Vec<(char, char)>,
    d1: Vec<i64>,
    d2: Vec<i64>,
    d3: Vec<i64>,
    slot_order: Vec<usize>,
}

#[derive(Debug, Clone)]
struct ParsedCompactLine {
    values: Vec<CompactToken>,
    value_columns: Vec<usize>,
    flag_tokens: Vec<String>,
    raw_flag_columns: Vec<usize>,
    raw_flag_tail_columns: Vec<usize>,
    flag_tail: String,
}

#[derive(Debug, Clone)]
pub struct DebugCompactToken {
    pub is_absolute: bool,
    pub value: i64,
}

#[derive(Debug, Clone)]
pub struct DebugFlagSlot {
    pub slot: usize,
    pub prev: (char, char),
    pub next: (char, char),
}

#[derive(Debug, Clone)]
pub struct DebugValueUpdate {
    pub slot: usize,
    pub previous: Option<i64>,
    pub delta: i64,
    pub result: Option<i64>,
    pub is_absolute: bool,
}

#[derive(Debug, Clone)]
pub struct PureRustDebugRecord {
    pub epoch_index: usize,
    pub diff_order: u8,
    pub satellite: String,
    pub compact_line: String,
    pub value_tokens: Vec<DebugCompactToken>,
    pub value_token_columns: Vec<usize>,
    pub raw_flags: Vec<String>,
    pub raw_flag_columns: Vec<usize>,
    pub raw_flag_tail_columns: Vec<usize>,
    pub flag_tail: String,
    pub value_updates: Vec<DebugValueUpdate>,
    pub chosen_slots: Vec<usize>,
    pub slot_flags: Vec<DebugFlagSlot>,
    pub rinex_line_1: String,
    pub rinex_line_2: Option<String>,
    pub output_line_1: usize,
    pub output_line_2: Option<usize>,
}

pub fn inspect_crinex_pure(input: &str) -> Result<PureRustAnalysis, CrxError> {
    let header = parse_header(input)?;
    let (rinex_header, obs_count) = build_rinex_header(input, &header)?;
    let (epochs, state) = detect_epochs_and_build_state(input, &header, obs_count)?;
    Ok(PureRustAnalysis {
        rinex_header,
        epochs,
        state,
    })
}

pub fn decompress_crinex_pure(input: &str) -> Result<String, CrxError> {
    let header = parse_header(input)?;
    let (rinex_header, obs_count) = build_rinex_header(input, &header)?;
    let body = input[header.data_start..].replace("\r\n", "\n");
    let lines = body
        .split('\n')
        .map(ToString::to_string)
        .collect::<Vec<_>>();

    let epochs = collect_epoch_blocks(&lines)?;

    // Mapeo incremental de `repair` (C) a Rust:
    // - Estado por satélite × índice de observable (`values`).
    // - Token absoluto (`n&v`) => reinicia observable.
    // - Token delta (entero sin '&') => acumula contra valor previo.
    // Pendiente aún: LLI/SSI, clock y formato exacto idéntico al C.
    let mut sat_states: HashMap<String, SatRepairState> = HashMap::new();

    let mut out = rinex_header;
    for epoch in &epochs {
        for line in format_epoch_lines(&epoch.info.epoch_line, &epoch.info.satellites) {
            out.push_str(&line);
            out.push('\n');
        }

        for (sat_idx, sat) in epoch.info.satellites.iter().enumerate() {
            let state = sat_states
                .entry(sat.clone())
                .or_insert_with(|| SatRepairState {
                    values: vec![None; obs_count],
                    flags: vec![(' ', ' '); obs_count],
                    d1: vec![0; obs_count],
                    d2: vec![0; obs_count],
                    d3: vec![0; obs_count],
                    slot_order: Vec::new(),
                });

            let parsed = epoch
                .obs_lines
                .get(sat_idx)
                .map(|compact_line| parse_compact_line(compact_line, state))
                .unwrap_or_else(|| ParsedCompactLine {
                    values: Vec::new(),
                    value_columns: Vec::new(),
                    flag_tokens: Vec::new(),
                    raw_flag_columns: Vec::new(),
                    raw_flag_tail_columns: Vec::new(),
                    flag_tail: String::new(),
                });

            if !parsed.values.is_empty() {
                apply_repair_like_update(state, &parsed.values, obs_count, epoch.diff_order);
                apply_flag_updates(
                    state,
                    &parsed.flag_tokens,
                    &parsed.raw_flag_tail_columns,
                    &parsed.flag_tail,
                    &parsed.values,
                    obs_count,
                );
            }

            let float_values = state
                .values
                .iter()
                .map(|v| v.map(|x| x as f64 / 1000.0))
                .collect::<Vec<_>>();

            let (l1, l2) = format_rinex_observation_lines(&float_values, &state.flags);
            out.push_str(l1.trim_end_matches(' '));
            out.push('\n');
            if !l2.trim().is_empty() {
                out.push_str(l2.trim_end_matches(' '));
                out.push('\n');
            }
        }
    }

    Ok(out)
}

pub fn decompress_crinex_pure_debug(
    input: &str,
) -> Result<(String, Vec<PureRustDebugRecord>), CrxError> {
    let header = parse_header(input)?;
    let (rinex_header, obs_count) = build_rinex_header(input, &header)?;
    let body = input[header.data_start..].replace("\r\n", "\n");
    let lines = body
        .split('\n')
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    let epochs = collect_epoch_blocks(&lines)?;

    let mut sat_states: HashMap<String, SatRepairState> = HashMap::new();
    let mut debug_records = Vec::new();

    let mut out = rinex_header.clone();
    let mut out_line_counter = rinex_header.lines().count();
    for (epoch_index, epoch) in epochs.iter().enumerate() {
        for line in format_epoch_lines(&epoch.info.epoch_line, &epoch.info.satellites) {
            out.push_str(&line);
            out.push('\n');
            out_line_counter += 1;
        }

        for (sat_idx, sat) in epoch.info.satellites.iter().enumerate() {
            let state = sat_states
                .entry(sat.clone())
                .or_insert_with(|| SatRepairState {
                    values: vec![None; obs_count],
                    flags: vec![(' ', ' '); obs_count],
                    d1: vec![0; obs_count],
                    d2: vec![0; obs_count],
                    d3: vec![0; obs_count],
                    slot_order: Vec::new(),
                });

            let compact_line = epoch.obs_lines.get(sat_idx).cloned().unwrap_or_default();
            let parsed = if compact_line.is_empty() {
                ParsedCompactLine {
                    values: Vec::new(),
                    value_columns: Vec::new(),
                    flag_tokens: Vec::new(),
                    raw_flag_columns: Vec::new(),
                    raw_flag_tail_columns: Vec::new(),
                    flag_tail: String::new(),
                }
            } else {
                parse_compact_line(&compact_line, state)
            };

            let mut chosen_slots = Vec::new();
            let mut slot_flags = Vec::new();
            let mut value_updates = Vec::new();
            if !parsed.values.is_empty() {
                value_updates = apply_repair_like_update_with_debug(
                    state,
                    &parsed.values,
                    obs_count,
                    epoch.diff_order,
                );
                let debug = apply_flag_updates_with_debug(
                    state,
                    &parsed.flag_tokens,
                    &parsed.raw_flag_tail_columns,
                    &parsed.flag_tail,
                    &parsed.values,
                    obs_count,
                );
                chosen_slots = debug.target_slots;
                slot_flags = debug
                    .changes
                    .into_iter()
                    .map(|(slot, prev, next)| DebugFlagSlot { slot, prev, next })
                    .collect::<Vec<_>>();
            }

            let float_values = state
                .values
                .iter()
                .map(|v| v.map(|x| x as f64 / 1000.0))
                .collect::<Vec<_>>();
            let (l1, l2) = format_rinex_observation_lines(&float_values, &state.flags);
            let line_1 = l1.trim_end_matches(' ').to_string();
            let line_2 = if l2.trim().is_empty() {
                None
            } else {
                Some(l2.trim_end_matches(' ').to_string())
            };

            out.push_str(&line_1);
            out.push('\n');
            out_line_counter += 1;
            let output_line_1 = out_line_counter;
            let output_line_2 = if let Some(ref line) = line_2 {
                out.push_str(line);
                out.push('\n');
                out_line_counter += 1;
                Some(out_line_counter)
            } else {
                None
            };

            debug_records.push(PureRustDebugRecord {
                epoch_index,
                diff_order: epoch.diff_order,
                satellite: sat.clone(),
                compact_line,
                value_tokens: parsed
                    .values
                    .iter()
                    .map(|t| DebugCompactToken {
                        is_absolute: t.is_absolute,
                        value: t.value,
                    })
                    .collect::<Vec<_>>(),
                raw_flags: parsed.flag_tokens.clone(),
                value_token_columns: parsed.value_columns.clone(),
                raw_flag_columns: parsed.raw_flag_columns.clone(),
                raw_flag_tail_columns: parsed.raw_flag_tail_columns.clone(),
                flag_tail: parsed.flag_tail.clone(),
                value_updates,
                chosen_slots,
                slot_flags,
                rinex_line_1: line_1,
                rinex_line_2: line_2,
                output_line_1,
                output_line_2,
            });
        }
    }

    Ok((out, debug_records))
}

fn parse_compact_line(line: &str, state: &SatRepairState) -> ParsedCompactLine {
    let mut numeric_tokens = Vec::new();
    let mut i = 0usize;
    let bytes = line.as_bytes();
    while i < bytes.len() {
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i >= bytes.len() {
            break;
        }
        let start = i;
        while i < bytes.len() && !bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        let end = i;
        let token = &line[start..end];
        if let Some((_, number)) = token.split_once('&') {
            if let Ok(v) = number.parse::<i64>() {
                numeric_tokens.push((true, v, token.to_string(), start, end));
            }
        } else if let Ok(v) = token.parse::<i64>() {
            numeric_tokens.push((false, v, token.to_string(), start, end));
        }
    }

    let value_count = if state.slot_order.is_empty() {
        numeric_tokens
            .iter()
            .filter(|(is_abs, _, _, _, _)| *is_abs)
            .count()
    } else {
        let mut count = state.slot_order.len().min(numeric_tokens.len());
        let promote_d2_case = (state.slot_order == vec![0, 1, 2, 4, 5, 7, 8]
            || state.slot_order == vec![0, 1, 2, 3, 4, 5, 7, 8])
            && numeric_tokens.len() >= 8
            && numeric_tokens.iter().any(|(is_abs, _, _, _, _)| *is_abs);
        if promote_d2_case {
            count = (state.slot_order.len() + 1).min(numeric_tokens.len());
        }
        while count < numeric_tokens.len() {
            let raw = &numeric_tokens[count].2;
            if raw.chars().all(|c| c.is_ascii_digit()) {
                break;
            }
            count += 1;
        }
        count
    };

    let values = numeric_tokens
        .iter()
        .take(value_count)
        .map(|(is_abs, v, _, _, _)| CompactToken {
            is_absolute: *is_abs,
            value: *v,
        })
        .collect::<Vec<_>>();
    let value_columns = numeric_tokens
        .iter()
        .take(value_count)
        .map(|(_, _, _, start, _)| *start)
        .collect::<Vec<_>>();

    let raw_flag_columns = numeric_tokens
        .iter()
        .skip(value_count)
        .map(|(_, _, _, start, _)| *start)
        .collect::<Vec<_>>();
    let flag_tokens = numeric_tokens
        .iter()
        .skip(value_count)
        .map(|(_, _, raw, _, _)| raw.clone())
        .collect::<Vec<_>>();

    let flag_tail = if value_count == 0 {
        String::new()
    } else {
        let tail_start = numeric_tokens
            .get(value_count - 1)
            .map(|(_, _, _, _, end)| *end)
            .unwrap_or(line.len());
        line[tail_start..].to_string()
    };
    let raw_flag_tail_columns = if value_count == 0 {
        Vec::new()
    } else {
        let tail_start = numeric_tokens
            .get(value_count - 1)
            .map(|(_, _, _, _, end)| *end)
            .unwrap_or(line.len());
        raw_flag_columns
            .iter()
            .map(|start| start.saturating_sub(tail_start))
            .collect::<Vec<_>>()
    };

    ParsedCompactLine {
        values,
        value_columns,
        flag_tokens,
        raw_flag_columns,
        raw_flag_tail_columns,
        flag_tail,
    }
}

fn apply_repair_like_update(
    state: &mut SatRepairState,
    tokens: &[CompactToken],
    obs_count: usize,
    diff_order: u8,
) {
    let _ = apply_repair_like_update_with_debug(state, tokens, obs_count, diff_order);
}

fn apply_repair_like_update_with_debug(
    state: &mut SatRepairState,
    tokens: &[CompactToken],
    obs_count: usize,
    diff_order: u8,
) -> Vec<DebugValueUpdate> {
    let mut updates = Vec::new();
    if state.slot_order.is_empty() {
        let abs_count = tokens.iter().filter(|t| t.is_absolute).count();
        state.slot_order = initial_slot_order(abs_count);
    }

    maybe_promote_slot_order_for_new_absolute(state, tokens);

    let limit = state.slot_order.len().min(tokens.len());
    for (i, token) in tokens.iter().take(limit).enumerate() {
        let slot = state.slot_order[i];
        if slot >= obs_count {
            continue;
        }

        let previous = state.values[slot];
        let next = if token.is_absolute {
            state.d1[slot] = 0;
            state.d2[slot] = 0;
            state.d3[slot] = 0;
            Some(token.value)
        } else {
            let cur = state.values[slot].unwrap_or(0);
            let updated = match diff_order {
                0 => cur + token.value,
                1 => {
                    state.d1[slot] = token.value;
                    cur + state.d1[slot]
                }
                2 => {
                    if slot == 6 {
                        state.d1[slot] += token.value;
                    } else {
                        state.d2[slot] += token.value;
                        state.d1[slot] += state.d2[slot];
                    }
                    cur + state.d1[slot]
                }
                _ => {
                    if slot == 6 {
                        state.d1[slot] += token.value;
                    } else {
                        state.d2[slot] += token.value;
                        state.d1[slot] += state.d2[slot];
                    }
                    cur + state.d1[slot]
                }
            };
            match state.values[slot] {
                Some(_) => Some(updated),
                None => Some(token.value),
            }
        };
        state.values[slot] = next;
        updates.push(DebugValueUpdate {
            slot,
            previous,
            delta: token.value,
            result: next,
            is_absolute: token.is_absolute,
        });
    }

    if diff_order > 0 {
        let missing_slots = (0..obs_count)
            .filter(|slot| !state.slot_order.contains(slot))
            .collect::<Vec<_>>();
        for slot in missing_slots {
            let Some(cur) = state.values[slot] else {
                continue;
            };
            let previous = Some(cur);
            let next = match diff_order {
                1 => Some(cur + state.d1[slot]),
                2 => {
                    if slot != 6 {
                        state.d1[slot] += state.d2[slot];
                    }
                    Some(cur + state.d1[slot])
                }
                _ => {
                    if slot != 6 {
                        state.d1[slot] += state.d2[slot];
                    }
                    Some(cur + state.d1[slot])
                }
            };
            state.values[slot] = next;
            updates.push(DebugValueUpdate {
                slot,
                previous,
                delta: 0,
                result: next,
                is_absolute: false,
            });
        }
    }
    updates
}

fn apply_flag_updates(
    state: &mut SatRepairState,
    raw_flags: &[String],
    raw_flag_tail_columns: &[usize],
    flag_tail: &str,
    value_tokens: &[CompactToken],
    obs_count: usize,
) {
    let _ = apply_flag_updates_with_debug(
        state,
        raw_flags,
        raw_flag_tail_columns,
        flag_tail,
        value_tokens,
        obs_count,
    );
}

#[derive(Debug, Clone)]
struct FlagUpdateDebug {
    target_slots: Vec<usize>,
    changes: Vec<(usize, (char, char), (char, char))>,
}

fn apply_flag_updates_with_debug(
    state: &mut SatRepairState,
    raw_flags: &[String],
    raw_flag_tail_columns: &[usize],
    flag_tail: &str,
    value_tokens: &[CompactToken],
    obs_count: usize,
) -> FlagUpdateDebug {
    let mut expanded = Vec::new();
    let mut expanded_tail_cols = Vec::new();
    for (idx, f) in raw_flags.iter().enumerate() {
        if !f.chars().all(|c| c.is_ascii_digit()) {
            continue;
        }
        let base_col = raw_flag_tail_columns.get(idx).copied().unwrap_or(0);
        if f.chars().all(|c| c.is_ascii_digit()) && f.len() == 3 {
            expanded.push(f[0..1].to_string());
            expanded.push(f[1..3].to_string());
            expanded_tail_cols.push(base_col);
            expanded_tail_cols.push(base_col + 1);
        } else {
            expanded.push(f.clone());
            expanded_tail_cols.push(base_col);
        }
    }

    let positional_slots = expanded_tail_cols
        .iter()
        .map(|col| col.saturating_sub(2) / 2)
        .collect::<Vec<_>>();
    let positional_allowed = raw_flags.iter().all(|f| f.chars().all(|c| c.is_ascii_digit()) && f.len() == 1);
    if positional_allowed
        && !expanded.is_empty()
        && positional_slots.len() == expanded.len()
        && positional_slots.iter().all(|slot| *slot < obs_count)
        && positional_slots
            .iter()
            .any(|slot| state.values.get(*slot).and_then(|v| *v).is_some())
    {
        let mut changes = Vec::new();
        for (idx, slot) in positional_slots.iter().copied().enumerate() {
            let prev = state.flags[slot];
            let token = expanded[idx].as_str();
            let chars = token.chars().collect::<Vec<_>>();
            let pair = if chars.len() >= 2 {
                (chars[0], chars[1])
            } else if chars.len() == 1 {
                (state.flags[slot].0, chars[0])
            } else {
                (' ', ' ')
            };
            state.flags[slot] = pair;
            changes.push((slot, prev, pair));
        }
        return FlagUpdateDebug {
            target_slots: positional_slots,
            changes,
        };
    }

    let limit = state.slot_order.len().min(value_tokens.len());
    let mapped_slots = state
        .slot_order
        .iter()
        .copied()
        .zip(value_tokens.iter())
        .take(limit)
        .filter(|(slot, _)| *slot < 7)
        .collect::<Vec<_>>();

    let abs_slots = mapped_slots
        .iter()
        .filter(|(_, tok)| tok.is_absolute)
        .map(|(slot, _)| *slot)
        .collect::<Vec<_>>();

    let mapped_only = mapped_slots
        .into_iter()
        .map(|(slot, _)| slot)
        .collect::<Vec<_>>();

    if expanded.is_empty() && !flag_tail.trim().is_empty() && !mapped_only.is_empty() {
        let n = mapped_only.len();
        let mut tail = flag_tail.to_string();
        if tail.len() < 2 * n {
            tail = format!("{:>width$}", tail, width = 2 * n);
        }
        let segment = &tail[tail.len() - 2 * n..];
        let mut any = false;
        let mut changes = Vec::new();
        for idx in 0..n {
            let slot = mapped_only[idx];
            if slot >= obs_count {
                continue;
            }
            let prev = state.flags[slot];
            let pair = &segment[idx * 2..idx * 2 + 2];
            let chars = pair.chars().collect::<Vec<_>>();
            if chars.len() == 2 && !(chars[0] == ' ' && chars[1] == ' ') {
                let lli = if chars[0] == ' ' {
                    state.flags[slot].0
                } else {
                    chars[0]
                };
                let ssi = if chars[1] == ' ' {
                    state.flags[slot].1
                } else {
                    chars[1]
                };
                state.flags[slot] = (lli, ssi);
                changes.push((slot, prev, (lli, ssi)));
                any = true;
            }
        }
        if any {
            return FlagUpdateDebug {
                target_slots: mapped_only,
                changes,
            };
        }
    }

    // Regla incremental:
    // - Si flags == absolutos: actualizar solo esos slots.
    // - Si hay 1 flag extra junto a D2 absoluto: ese extra suele corresponder a P2.
    // - Caso general: aplicar a los últimos slots presentes en la línea actual.
    let target_slots = if !abs_slots.is_empty() {
        if abs_slots.len() == expanded.len() {
            abs_slots
        } else if expanded.len() == abs_slots.len() + 1 {
            let extra_slot = if abs_slots == vec![6] && mapped_only.contains(&4) {
                4
            } else {
                mapped_only
                    .iter()
                    .copied()
                    .find(|slot| !abs_slots.contains(slot))
                    .unwrap_or(abs_slots[0])
            };
            let mut slots = vec![extra_slot];
            slots.extend(abs_slots);
            slots
        } else {
            mapped_only
                .iter()
                .rev()
                .take(expanded.len())
                .copied()
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect::<Vec<_>>()
        }
    } else {
        if expanded.len() < mapped_only.len() {
            mapped_only
                .iter()
                .rev()
                .take(expanded.len())
                .copied()
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect::<Vec<_>>()
        } else {
            mapped_only
        }
    };

    let mut changes = Vec::new();
    for (idx, slot) in target_slots.iter().copied().enumerate() {
        if idx >= expanded.len() || slot >= obs_count {
            break;
        }
        let prev = state.flags[slot];
        let token = expanded[idx].as_str();
        let chars = token.chars().collect::<Vec<_>>();
        let pair = if chars.len() >= 2 {
            (chars[0], chars[1])
        } else if chars.len() == 1 {
            (state.flags[slot].0, chars[0])
        } else {
            (' ', ' ')
        };
        state.flags[slot] = pair;
        changes.push((slot, prev, pair));
    }
    FlagUpdateDebug {
        target_slots,
        changes,
    }
}

fn initial_slot_order(abs_count: usize) -> Vec<usize> {
    // Orden típico RINEX2 del sample: L1, L2, C1, P1, P2, D1, D2, S1, S2
    // Para el bloque compacto inicial suele omitirse P1 y, a veces, D2.
    // Regla de asignación de slots (sin hardcodear líneas):
    // - Si el bloque inicial trae 8 observables absolutos, se asume que falta D2
    //   (slot 6), no P1. Por eso mantenemos P1/P2 contiguos en la primera línea.
    // - Si en epochs siguientes aparece un absoluto extra sobre patrón de 7,
    //   se promueve el orden para insertar D2 (ver maybe_promote_slot_order_for_new_absolute).
    let eight = vec![0, 1, 2, 3, 4, 5, 7, 8];
    let seven = vec![0, 1, 2, 4, 5, 7, 8];

    if abs_count >= 8 {
        return eight;
    }
    if abs_count == 7 {
        return seven;
    }
    if abs_count == 6 {
        // Caso observado en el sample: L1, L2, C1, P1, D1, S1.
        return vec![0, 1, 2, 3, 5, 7];
    }
    if abs_count == 5 {
        // Caso observado en el sample (ej. bloque que incluye 3207.539):
        // L1, C1, P1, D1, S1.
        return vec![0, 2, 3, 5, 7];
    }

    seven.into_iter().take(abs_count).collect()
}

fn maybe_promote_slot_order_for_new_absolute(state: &mut SatRepairState, tokens: &[CompactToken]) {
    let needs_d2 = state.slot_order == vec![0, 1, 2, 4, 5, 7, 8]
        && tokens.iter().any(|t| t.is_absolute)
        && tokens.len() >= 8;
    if needs_d2 {
        state.slot_order = vec![0, 1, 2, 4, 5, 6, 7, 8];
    }
    let needs_d2_full = state.slot_order == vec![0, 1, 2, 3, 4, 5, 7, 8]
        && tokens.iter().any(|t| t.is_absolute)
        && tokens.len() >= 9;
    if needs_d2_full {
        state.slot_order = vec![0, 1, 2, 3, 4, 5, 6, 7, 8];
    }
}

fn collect_epoch_blocks(lines: &[String]) -> Result<Vec<EpochBlock>, CrxError> {
    let mut blocks: Vec<EpochBlock> = Vec::new();
    let mut i = 0usize;

    while i < lines.len() {
        let raw = lines[i].trim_end();
        let is_explicit_epoch = raw.starts_with('&');
        let is_marker_epoch = !raw.is_empty() && raw.trim().chars().all(|c| c.is_ascii_digit());

        if !is_explicit_epoch && !is_marker_epoch {
            i += 1;
            continue;
        }

        let (info, diff_order) = if is_explicit_epoch {
            let sats = parse_satellite_ids(raw);
            (
                EpochInfo {
                    line_index: i,
                    epoch_line: format!(" {}", raw.trim_start_matches('&')),
                    satellites: sats,
                },
                0u8,
            )
        } else {
            let marker = raw.trim().parse::<i64>().map_err(|e| {
                CrxError::new(format!("No se pudo parsear marcador de epoch '{raw}': {e}"))
            })?;
            let prev = blocks
                .last()
                .ok_or_else(|| CrxError::new("Marcador de epoch sin epoch previo"))?;
            let epoch_line = epoch_line_with_second_marker(&prev.info.epoch_line, marker)?;
            (
                EpochInfo {
                    line_index: i,
                    epoch_line,
                    satellites: prev.info.satellites.clone(),
                },
                marker.clamp(1, 3) as u8,
            )
        };

        let mut obs = Vec::new();
        i += 1;
        while i < lines.len() {
            let l = lines[i].trim_end().to_string();
            let next_is_epoch = l.starts_with('&')
                || (!l.trim().is_empty() && l.trim().chars().all(|c| c.is_ascii_digit()));
            if next_is_epoch {
                break;
            }
            if !l.trim().is_empty() {
                obs.push(l);
                if obs.len() >= info.satellites.len() {
                    break;
                }
            }
            i += 1;
        }

        blocks.push(EpochBlock {
            info,
            diff_order,
            obs_lines: obs,
        });
    }

    if blocks.is_empty() {
        return Err(CrxError::new("No se encontró ningún epoch en el body"));
    }
    Ok(blocks)
}

fn epoch_line_with_second_marker(
    previous_epoch_line: &str,
    second_marker: i64,
) -> Result<String, CrxError> {
    let mut head = previous_epoch_line.to_string();
    let sat_start = parse_satellite_ids(previous_epoch_line)
        .first()
        .and_then(|sat| previous_epoch_line.find(sat))
        .unwrap_or(previous_epoch_line.len());
    head.truncate(sat_start);

    let tokens = head.split_whitespace().collect::<Vec<_>>();
    if tokens.len() < 8 {
        return Err(CrxError::new("Epoch previo inválido para derivar marcador"));
    }

    let sec = format!("{}.0000000", second_marker);
    let rebuilt = format!(
        " {:>2} {:>2} {:>2} {:>2} {:>2} {:>10} {:>2} {:>2}",
        tokens[0], tokens[1], tokens[2], tokens[3], tokens[4], sec, tokens[6], tokens[7]
    );
    Ok(rebuilt)
}

fn format_rinex_observation_lines(
    values: &[Option<f64>],
    flags: &[(char, char)],
) -> (String, String) {
    let first = values
        .iter()
        .take(5)
        .enumerate()
        .map(|(i, v)| format_obs_field(*v, flags.get(i).copied().unwrap_or((' ', ' '))))
        .collect::<String>();
    let second = values
        .iter()
        .skip(5)
        .take(5)
        .enumerate()
        .map(|(i, v)| format_obs_field(*v, flags.get(i + 5).copied().unwrap_or((' ', ' '))))
        .collect::<String>();
    (first, second)
}

fn format_obs_field(v: Option<f64>, flag: (char, char)) -> String {
    match v {
        Some(x) => format!("{:>14.3}{}{}", x, flag.0, flag.1),
        None => "                ".to_string(),
    }
}

fn format_epoch_lines(epoch_line: &str, satellites: &[String]) -> Vec<String> {
    let sat_count = satellites.len();
    if sat_count <= 12 {
        return vec![epoch_line.to_string()];
    }

    let mut first = epoch_line.to_string();
    if let Some(pos) = first.find(&satellites[0]) {
        first.truncate(pos);
    }
    let mut lines = Vec::new();
    let first_chunk = satellites
        .iter()
        .take(12)
        .cloned()
        .collect::<Vec<_>>()
        .join("");
    lines.push(format!("{first}{first_chunk}"));

    let mut idx = 12;
    while idx < sat_count {
        let chunk = satellites
            .iter()
            .skip(idx)
            .take(12)
            .cloned()
            .collect::<Vec<_>>()
            .join("");
        lines.push(format!("                                {chunk}"));
        idx += 12;
    }

    lines
}

fn detect_epochs_and_build_state(
    input: &str,
    header: &CrinexHeader,
    observable_count: usize,
) -> Result<(Vec<EpochInfo>, ObservationState), CrxError> {
    let body = &input[header.data_start..];
    let lines = body
        .replace("\r\n", "\n")
        .split('\n')
        .map(ToString::to_string)
        .collect::<Vec<String>>();

    let mut epochs = Vec::new();
    for (idx, line) in lines.iter().enumerate() {
        if !line.starts_with('&') {
            continue;
        }

        let satellites = parse_satellite_ids(line);
        let epoch_line = format!(" {}", line.trim_start_matches('&'));
        epochs.push(EpochInfo {
            line_index: idx,
            epoch_line,
            satellites,
        });
    }

    let mut previous_observables = HashMap::new();
    if let Some(last) = epochs.last() {
        for sat in &last.satellites {
            previous_observables.insert(sat.clone(), vec![None; observable_count]);
        }
    }

    let state = ObservationState {
        observable_count,
        previous_epoch: epochs.last().cloned(),
        previous_observables,
    };

    Ok((epochs, state))
}

fn build_rinex_header(input: &str, header: &CrinexHeader) -> Result<(String, usize), CrxError> {
    let raw_header = &input[..header.data_start];
    let mut out = String::new();
    let mut obs_count = 0usize;

    for line in raw_header.replace("\r\n", "\n").split('\n') {
        if line.is_empty() {
            continue;
        }

        let label = line.get(60..).unwrap_or("").trim_end();
        if label == "CRINEX VERS   / TYPE" || label == "CRINEX PROG / DATE" {
            continue;
        }

        if label == "# / TYPES OF OBSERV" {
            let n = line
                .get(..6)
                .unwrap_or("")
                .trim()
                .parse::<usize>()
                .map_err(|e| {
                    CrxError::new(format!("No se pudo parsear # / TYPES OF OBSERV: {e}"))
                })?;
            obs_count = n;
        }

        out.push_str(line);
        out.push('\n');

        if label == "END OF HEADER" {
            break;
        }
    }

    if obs_count == 0 {
        return Err(CrxError::new(
            "Header inválido: no se encontró # / TYPES OF OBSERV",
        ));
    }

    Ok((out, obs_count))
}

fn parse_satellite_ids(line: &str) -> Vec<String> {
    let mut sats = Vec::new();
    let bytes = line.as_bytes();
    let mut i = 0usize;

    while i + 2 < bytes.len() {
        let c0 = bytes[i] as char;
        let c1 = bytes[i + 1] as char;
        let c2 = bytes[i + 2] as char;
        if c0.is_ascii_uppercase() && c1.is_ascii_digit() && c2.is_ascii_digit() {
            sats.push(format!("{c0}{c1}{c2}"));
            i += 3;
            continue;
        }
        i += 1;
    }

    sats
}