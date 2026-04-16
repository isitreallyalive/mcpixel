const DEFAULT_SIZE: u32 = 32;
const DEFAULT_STRETCH: bool = false;
const DEFAULT_COLOURS: u32 = 256;
const DEFAULT_BRIGHTNESS: f32 = 1.2;
const DEFAULT_SATURATION: f32 = 1.2;
const DEFAULT_SMOOTHING: f32 = 0.3;
const DEFAULT_OVERLAY: bool = false;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen::prelude::wasm_bindgen)]
#[cfg_attr(feature = "clap", derive(clap::Args, Debug))]
pub struct Configuration {
    /// Maximum width or height in blocks
    #[cfg_attr(feature = "clap", arg(
        short, long,
        default_value_t = DEFAULT_SIZE
    ))]
    pub size: u32,
    /// Stretch image to fit the size exactly (may distort)
    #[cfg_attr(feature = "clap", arg(
        long,
        default_value_t = DEFAULT_STRETCH
    ))]
    pub stretch: bool,
    /// Number of colours to use
    #[cfg_attr(feature = "clap", arg(
        short = 'c', long,
        default_value_t = DEFAULT_COLOURS
    ))]
    pub colours: u32,
    /// Adjust overall brightness
    #[cfg_attr(feature = "clap", arg(
        short = 'b', long,
        default_value_t = DEFAULT_BRIGHTNESS
    ))]
    pub brightness: f32,
    /// Adjust colour intensity
    #[cfg_attr(feature = "clap", arg(
        long,
        default_value_t = DEFAULT_SATURATION
    ))]
    pub saturation: f32,
    /// Reduce harsh colour changes
    #[cfg_attr(feature = "clap", arg(
        long = "smooth",
        default_value_t = DEFAULT_SMOOTHING
    ))]
    pub smoothing: f32,
    /// Add glass layer to blend colours more smoothly
    #[cfg_attr(feature = "clap", arg(
        short = 'o', long,
        default_value_t = DEFAULT_OVERLAY
    ))]
    pub overlay: bool,
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            size: DEFAULT_SIZE,
            stretch: DEFAULT_STRETCH,
            colours: DEFAULT_COLOURS,
            brightness: DEFAULT_BRIGHTNESS,
            saturation: DEFAULT_SATURATION,
            smoothing: DEFAULT_SMOOTHING,
            overlay: DEFAULT_OVERLAY,
        }
    }
}

#[cfg(target_arch = "wasm32")]
mod wasm {
    use super::*;
    use wasm_bindgen::prelude::*;

    #[derive(serde::Deserialize)]
    struct ConfigurationInput {
        #[serde(default)]
        size: Option<u32>,
        #[serde(default)]
        stretch: Option<bool>,
        #[serde(default)]
        colours: Option<u32>,
        #[serde(default)]
        brightness: Option<f32>,
        #[serde(default)]
        saturation: Option<f32>,
        #[serde(default)]
        smoothing: Option<f32>,
        #[serde(default)]
        overlay: Option<bool>,
    }

    #[wasm_bindgen]
    impl Configuration {
        #[wasm_bindgen(constructor)]
        pub fn new(obj: JsValue) -> Result<Configuration, JsValue> {
            let input: ConfigurationInput = serde_wasm_bindgen::from_value(obj)
                .map_err(|_| JsValue::from_str("invalid configuration"))?;

            Ok(Self {
                size: input.size.unwrap_or(DEFAULT_SIZE),
                stretch: input.stretch.unwrap_or(DEFAULT_STRETCH),
                colours: input.colours.unwrap_or(DEFAULT_COLOURS),
                brightness: input.brightness.unwrap_or(DEFAULT_BRIGHTNESS),
                saturation: input.saturation.unwrap_or(DEFAULT_SATURATION),
                smoothing: input.smoothing.unwrap_or(DEFAULT_SMOOTHING),
                overlay: input.overlay.unwrap_or(DEFAULT_OVERLAY),
            })
        }
    }
}
