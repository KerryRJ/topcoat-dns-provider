//! Optional font for hosts that want a consistent default typography.

/// Geist, served using the host application's Topcoat asset configuration.
pub const GEIST: topcoat::font::Font =
    topcoat::font::fontsource::fontsource_font!(GEIST, host: Asset);
