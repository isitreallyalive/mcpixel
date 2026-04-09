const DEFAULT_SIZE: u32 = 32;
const DEFAULT_STRETCH: bool = false;
const DEFAULT_COLOURS: u32 = 256;
const DEFAULT_BRIGHTNESS: f32 = 1.2;
const DEFAULT_SATURATION: f32 = 1.2;
const DEFAULT_SMOOTHING: f32 = 0.3;
const DEFAULT_OVERLAY: bool = false;

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
