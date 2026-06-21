// trade/vsta/src/dat.rs
//
// Emerald type definitions - canonical source of truth for all emerald data.
// Used by EmeraldCollectionControl, MakeSwatV, MakeSallyV, and v_dr_r wiring.
//
// TWO enums:
//   EmeraldColor  - the 15 colors, each has a chaos ticker and a super ticker
//   EmeraldTypes  - the display/selection type: Master, Super(color), Chaos(color), Random(str)
//
// JPG naming conventions (individual per-emerald images already exist):
//   Master     =>  ../jpg/master_0ES.jpg
//   Super      =>  ../jpg/super_0MES.jpg  ..super_1VIX.jpg  etc.
//   Chaos      =>  ../jpg/chaos_0_SPY.jpg ..chaos_1_TSLA.jpg etc.
//   Random     =>  ../jpg/default.jpg
//
// NOTE: there is always a ../jpg/chaos_green.jpg, super_purple.jpg, etc. you can rely on
// if you need to add new controls and don't know the exact individual file name.

// ============================================================
// EmeraldColor - the 15 canonical colors
// ============================================================

/// The 15 emerald colors.  Each maps to exactly one Chaos ticker (stock)
/// and one Super ticker (index/future).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EmeraldColor {
    Green,   // Chaos => SPY   // Super => /MES
    White,   // Chaos => TSLA  // Super => VIX
    Pink,    // Chaos => BRKB  // Super => SPX
    Purple,  // Chaos => NVDA  // Super => XSP
    Blue,    // Chaos => COST  // Super => BTC/USD
    Red,     // Chaos => SOXL  // Super => SOX
    Yellow,  // Chaos => WY    // Super => XAU (gold)
    Silver,    // Chaos => VTRS  // Super => RUT (russell 2k)
    Teal,    // Chaos => HP    // Super => DJX (dow jones)
    Orange,  // Chaos => QQQ   // Super => EXO (europe)
    Opal,    // Chaos => SEV   // Super => /NQ
    Wood,    // Chaos => SANA  // Super => NQ
    Copper,   // Chaos => VTWO  // Super => HGX (housing sector)
    Black,   // Chaos => NVTS  // Super => OIL (/CL)
    Gold,    // Chaos => LULU  // Super => Gold (/BTC)
}

impl EmeraldColor {
    pub fn chaos_ticker(&self) -> &'static str {
        match self {
            EmeraldColor::Green  => "SPY",
            EmeraldColor::White  => "TSLA",
            EmeraldColor::Pink   => "BRKB",
            EmeraldColor::Purple => "NVDA",
            EmeraldColor::Blue   => "COST",
            EmeraldColor::Red    => "SOXL",
            EmeraldColor::Yellow => "WY",
            EmeraldColor::Silver => "VTRS",
            EmeraldColor::Teal   => "HP",
            EmeraldColor::Orange => "QQQ",
            EmeraldColor::Opal   => "SEV",
            EmeraldColor::Wood   => "SANA",
            EmeraldColor::Copper => "VTWO",
            EmeraldColor::Black  => "NVTS",
            EmeraldColor::Gold   => "LULU",
        }
    }

    pub fn super_ticker(&self) -> &'static str {
        match self {
            EmeraldColor::Green  => "/MES",
            EmeraldColor::White  => "VIX",
            EmeraldColor::Pink   => "SPX",
            EmeraldColor::Purple => "XSP",
            EmeraldColor::Blue   => "BTC/USD",
            EmeraldColor::Red    => "SOX",
            EmeraldColor::Yellow => "XAU",
            EmeraldColor::Silver => "RUT",
            EmeraldColor::Teal   => "DJX",
            EmeraldColor::Orange => "EXO",
            EmeraldColor::Opal   => "/NQ",
            EmeraldColor::Wood   => "HGX",
            EmeraldColor::Copper => "/HG",  // copper 
            EmeraldColor::Black  => "/CL",  // oil
            EmeraldColor::Gold   => "/GC",  // gold
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            EmeraldColor::Green  => "Green",
            EmeraldColor::White  => "White",
            EmeraldColor::Pink   => "Pink",
            EmeraldColor::Purple => "Purple",
            EmeraldColor::Blue   => "Blue",
            EmeraldColor::Red    => "Red",
            EmeraldColor::Yellow => "Yellow",
            EmeraldColor::Silver => "Silver",
            EmeraldColor::Teal   => "Teal",
            EmeraldColor::Orange => "Orange",
            EmeraldColor::Opal   => "Opal",
            EmeraldColor::Wood   => "Wood",
            EmeraldColor::Copper  => "Copper",
            EmeraldColor::Black  => "Black",
            EmeraldColor::Gold   => "Gold",
        }
    }

    pub fn all_colors() -> &'static [EmeraldColor] {
        &[
            EmeraldColor::Green,
            EmeraldColor::White,
            EmeraldColor::Pink,
            EmeraldColor::Purple,
            EmeraldColor::Blue,
            EmeraldColor::Red,
            EmeraldColor::Yellow,
            EmeraldColor::Silver,
            EmeraldColor::Teal,
            EmeraldColor::Orange,
            EmeraldColor::Opal,
            EmeraldColor::Wood,
            EmeraldColor::Copper,
            EmeraldColor::Black,
            EmeraldColor::Gold,
        ]
    }

    /// Individual JPG handle for Chaos variant of this color.
    /// Files follow pattern: ../jpg/chaos_{index}_{TICKER}.jpg
    pub fn chaos_icon_handle(&self) -> iced::widget::image::Handle 
    { match self 
      {
            EmeraldColor::White  => iced::widget::image::Handle::from_bytes(include_bytes!("../jpg/chaos_white.jpg").to_vec()),
            EmeraldColor::Pink   => iced::widget::image::Handle::from_bytes(include_bytes!("../jpg/chaos_pink.jpg").to_vec()),
            EmeraldColor::Red    => iced::widget::image::Handle::from_bytes(include_bytes!("../jpg/chaos_red.jpg").to_vec()),
            EmeraldColor::Orange => iced::widget::image::Handle::from_bytes(include_bytes!("../jpg/chaos_orange.jpg").to_vec()),
            EmeraldColor::Yellow => iced::widget::image::Handle::from_bytes(include_bytes!("../jpg/chaos_yellow.jpg").to_vec()),
            EmeraldColor::Green  => iced::widget::image::Handle::from_bytes(include_bytes!("../jpg/chaos_green.jpg").to_vec()),
            EmeraldColor::Teal   => iced::widget::image::Handle::from_bytes(include_bytes!("../jpg/chaos_teal.jpg").to_vec()),
            EmeraldColor::Blue   => iced::widget::image::Handle::from_bytes(include_bytes!("../jpg/chaos_blue.jpg").to_vec()),
            EmeraldColor::Purple => iced::widget::image::Handle::from_bytes(include_bytes!("../jpg/chaos_purple.jpg").to_vec()),
            EmeraldColor::Black  => iced::widget::image::Handle::from_bytes(include_bytes!("../jpg/chaos_black.jpg").to_vec()),
            EmeraldColor::Gold   => iced::widget::image::Handle::from_bytes(include_bytes!("../jpg/chaos_gold.jpg").to_vec()),
            EmeraldColor::Silver => iced::widget::image::Handle::from_bytes(include_bytes!("../jpg/chaos_silver.jpg").to_vec()),
            EmeraldColor::Copper => iced::widget::image::Handle::from_bytes(include_bytes!("../jpg/chaos_copper.jpg").to_vec()),
            EmeraldColor::Wood   => iced::widget::image::Handle::from_bytes(include_bytes!("../jpg/chaos_wood.jpg").to_vec()),
            EmeraldColor::Opal   => iced::widget::image::Handle::from_bytes(include_bytes!("../jpg/chaos_opal.jpg").to_vec()),
        }
    }

    /// Individual JPG handle for Super variant of this color.
    /// Files follow pattern: ../jpg/super_{index}{TICKER}.jpg
    pub fn super_icon_handle(&self) -> iced::widget::image::Handle {
        match self {
            EmeraldColor::Green  => iced::widget::image::Handle::from_bytes(include_bytes!("../jpg/super_0MES.jpg").to_vec()),
            EmeraldColor::White  => iced::widget::image::Handle::from_bytes(include_bytes!("../jpg/super_1VIX.jpg").to_vec()),
            EmeraldColor::Pink   => iced::widget::image::Handle::from_bytes(include_bytes!("../jpg/super_2SPX.jpg").to_vec()),
            EmeraldColor::Purple => iced::widget::image::Handle::from_bytes(include_bytes!("../jpg/super_3XSP.jpg").to_vec()),
            EmeraldColor::Blue   => iced::widget::image::Handle::from_bytes(include_bytes!("../jpg/super_4BTC.jpg").to_vec()),
            EmeraldColor::Red    => iced::widget::image::Handle::from_bytes(include_bytes!("../jpg/super_5SOX.jpg").to_vec()),
            EmeraldColor::Yellow => iced::widget::image::Handle::from_bytes(include_bytes!("../jpg/super_6XAU.jpg").to_vec()),
            EmeraldColor::Silver => iced::widget::image::Handle::from_bytes(include_bytes!("../jpg/super_7RUT.jpg").to_vec()),
            EmeraldColor::Teal   => iced::widget::image::Handle::from_bytes(include_bytes!("../jpg/super_8DJX.jpg").to_vec()),
            EmeraldColor::Orange => iced::widget::image::Handle::from_bytes(include_bytes!("../jpg/super_9XEO.jpg").to_vec()),
            EmeraldColor::Opal   => iced::widget::image::Handle::from_bytes(include_bytes!("../jpg/super_aHGX.jpg").to_vec()),
            EmeraldColor::Wood   => iced::widget::image::Handle::from_bytes(include_bytes!("../jpg/super_bOIL.jpg").to_vec()),
            EmeraldColor::Copper => iced::widget::image::Handle::from_bytes(include_bytes!("../jpg/chaos_0_SPY.jpg").to_vec()),
            EmeraldColor::Black  => iced::widget::image::Handle::from_bytes(include_bytes!("../jpg/chaos_0_SPY.jpg").to_vec()),
            EmeraldColor::Gold   => iced::widget::image::Handle::from_bytes(include_bytes!("../jpg/chaos_0_SPY.jpg").to_vec()),
        }
    }
}

impl std::fmt::Display for EmeraldColor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

// ============================================================
// EmeraldTypes - the selection/display category
// ============================================================

/// EmeraldTypes - the four categories a user can select.
/// This is the primary type used throughout the UI and bot wiring.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EmeraldTypes {
    /// The Master Emerald: /ES futures (month-coded at runtime).
    /// button image ../jpg/master_0ES.jpg
    MasterEmerald,

    /// One of the 15 Super Emeralds (indices / futures / BTC).
    /// button image ../jpg/super_{index}{TICKER}.jpg  (individual per-color)
    SuperEmerald(EmeraldColor),

    /// One of the 15 Chaos Emeralds (individual stocks).
    /// button image ../jpg/chaos_{index}_{TICKER}.jpg  (individual per-color)
    ChaosEmerald(EmeraldColor),

    /// User-typed / FinAss-derived custom symbol.
    /// button image ../jpg/default.jpg
    RandomJewels(String),
}

impl EmeraldTypes {
    /// JPG icon handle for button display.
    pub fn button_icon_handle(&self) -> iced::widget::image::Handle {
        match self {
            EmeraldTypes::MasterEmerald =>
                iced::widget::image::Handle::from_bytes(include_bytes!("../jpg/master_0ES.jpg").to_vec()),
            EmeraldTypes::SuperEmerald(color) =>
                color.super_icon_handle(),
            EmeraldTypes::ChaosEmerald(color) =>
                color.chaos_icon_handle(),
            EmeraldTypes::RandomJewels(_) =>
                iced::widget::image::Handle::from_bytes(include_bytes!("../jpg/chaos_coral.jpg").to_vec()),
        }
    }

    /// The primary ticker this emerald tracks.
    /// For Master: month-coded /ES future (e.g. "/ESH5").
    /// For Super/Chaos: the static ticker string.
    /// For Random: the user-supplied string.
    pub fn base_ticker(&self) -> String {
        match self {
            EmeraldTypes::MasterEmerald => {
                use chrono::Datelike;
                let now = chrono::Utc::now();
                let month = now.month();
                let year = now.year() % 10;
                let month_code = match month {
                    1 | 2 | 3   => 'H',
                    4 | 5 | 6   => 'M',
                    7 | 8 | 9   => 'U',
                    10 | 11 | 12 => 'Z',
                    _ => 'H',
                };
                format!("/ES{}{}", month_code, year)
            }
            EmeraldTypes::SuperEmerald(color) => color.super_ticker().to_string(),
            EmeraldTypes::ChaosEmerald(color) => color.chaos_ticker().to_string(),
            EmeraldTypes::RandomJewels(name)  => name.clone(),
        }
    }

    /// Display label for UI and bot friendly names.
    pub fn label(&self) -> String {
        match self {
            EmeraldTypes::MasterEmerald             => "Master Emerald".to_string(),
            EmeraldTypes::SuperEmerald(color) => format!("Super {}", color.name()),
            EmeraldTypes::ChaosEmerald(color) => format!("Chaos {}", color.name()),
            EmeraldTypes::RandomJewels(name)  => format!("Jewels: {}", name),
        }
    }
}

impl std::fmt::Display for EmeraldTypes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label())
    }
}

// ============================================================
// SelectedAssets  (used by MakeSwatV, MakeSallyV)
// ============================================================

/// Holds the user's current selections for bot wiring.
/// `selected_emerald_type` replaces the old `selected_emerald: Option<EmeraldColor>`.
#[derive(Clone, Debug, Default)]
pub struct SelectedAssets {
    pub selected_emerald_type: Option<EmeraldTypes>,
    pub selected_finass: Option<dsta::FinAss>,
}