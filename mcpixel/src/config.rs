macro_rules! config {
    ($(
        $name:ident $([$long:expr])? $(($short:expr))?: $type:ty = $default:expr
    ),*) => {
        #[cfg_attr(feature = "clap", derive(clap::Args, Debug))]
        pub struct Configuration {
            $(
                #[cfg_attr(feature = "clap", arg(
                    long$( = $long)?, $(short = $short,)?
                    default_value_t = $default
                ))]
                pub $name: $type
            ),*
        }

        impl Default for Configuration {
            fn default() -> Self {
                Self {
                    $($name: $default),*
                }
            }
        }

        // js input type
        #[cfg(target_arch = "wasm32")]
        #[derive(tsify::Tsify, serde::Deserialize)]
        #[tsify(from_wasm_abi)]
        pub struct Config {
            $($name: Option<$type>),*
        }

        #[cfg(target_arch = "wasm32")]
        impl From<Config> for Configuration {
            fn from(config: Config) -> Self {
                Self {
                    $($name: config.$name.unwrap_or($default)),*
                }
            }
        }
    };
}

config! {
    size ('s'): u32 = 32,
    stretch: bool = false,
    colours ('c'): u32 = 256,
    brightness ('b'): f32 = 1.2,
    saturation: f32 = 1.2,
    smoothing ["smooth"]: f32 = 0.3,
    overlay ('o'): bool = false
}
