use engine::{simulate_wave, DungeonState, WaveConfig, ENGINE_VERSION};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn simulate_wave_wasm(
    dungeon: JsValue,
    wave: JsValue,
    seed: u64,
    max_ticks: u32,
) -> Result<JsValue, JsValue> {
    let dungeon: DungeonState = serde_wasm_bindgen::from_value(dungeon)
        .map_err(|err| JsValue::from_str(&format!("failed to parse dungeon: {err}")))?;
    let wave: WaveConfig = serde_wasm_bindgen::from_value(wave)
        .map_err(|err| JsValue::from_str(&format!("failed to parse wave: {err}")))?;

    simulate_wave(dungeon, wave, seed, max_ticks)
        .map_err(|err| JsValue::from_str(&err.to_string()))
        .and_then(|result| serde_wasm_bindgen::to_value(&result)
            .map_err(|err| JsValue::from_str(&format!("failed to serialize result: {err}"))))
}

#[wasm_bindgen]
pub fn engine_version_wasm() -> String {
    ENGINE_VERSION.to_string()
}
