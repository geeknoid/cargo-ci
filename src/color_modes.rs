use clap::{Parser, ValueEnum};

#[derive(Parser, ValueEnum, Debug, Clone, Copy, Default)]
pub enum ColorModes {
    #[default]
    Auto,
    Always,
    Never,
}
