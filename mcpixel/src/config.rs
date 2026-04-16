macro_rules! config {
    ($(
        $name:ident: $type:ty = $default:expr
    ),*) => {
        #[cfg_attr(feature = "clap", derive(clap::Args, Debug))]
        pub struct Configuration {
            $(pub $name: $type),*
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
    size: u32 = 32,
    stretch: bool = false,
    colours: u32 = 256,
    brightness: f32 = 1.2,
    saturation: f32 = 1.2,
    smoothing: f32 = 0.3,
    overlay: bool = false
}
