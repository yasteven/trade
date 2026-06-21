// trade/vsta/src/core/v_sally_preset/mod.rs
// (or split into state.rs + view.rs as your project structure requires)
//
// Context-aware Sally presets:
//   BTC/USD:  Tiny ($0.69), Small ($1.69), Medium ($10.69), Large ($100.69)
//   Stock:    1 share, 10 shares, 100 shares (at current ask)
//   Option:   1 contract, 10 contracts
//   Unknown:  show all

// ============================================================
// Preset Option enum
// ============================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SallyPresetOption {
    // BTC nominal dollar amounts → qty = dollars / ask_price
    TinyBTC,    // $0.69 worth
    SmallBTC,   // $1.69 worth
    MediumBTC,  // $10.69 worth
    LargeBTC,   // $100.69 worth

    // Stock share quantities
    OneShare,
    TenShares,
    HundredShares,

    // Option contracts (qty = contracts, price = mid)
    OneContract,
    TenContracts,
}

impl SallyPresetOption {
    pub fn label(&self) -> &'static str {
        match self {
            SallyPresetOption::TinyBTC       => "Tiny BTC\n($0.69)",
            SallyPresetOption::SmallBTC      => "Small BTC\n($1.69)",
            SallyPresetOption::MediumBTC     => "Medium BTC\n($10.69)",
            SallyPresetOption::LargeBTC      => "Large BTC\n($100.69)",
            SallyPresetOption::OneShare      => "1 Share",
            SallyPresetOption::TenShares     => "10 Shares",
            SallyPresetOption::HundredShares => "100 Shares",
            SallyPresetOption::OneContract   => "1 Contract",
            SallyPresetOption::TenContracts  => "10 Contracts",
        }
    }

    /// Convert to a HaggleMethod (sensible defaults per preset type)
    pub fn to_haggle_method(&self) -> dsta::HaggleMethod {
        let limits = dsta::HaggleLimits {
            retry_period: chrono::Duration::seconds(30),
            delta_choice:  0.01,
            delta_is_pct:  false,
            slippage_max:  0.10,
        };
        dsta::HaggleMethod::MinimumOfMidpointOrTheoreticalThenConcede(limits)
    }
}

impl std::fmt::Display for SallyPresetOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label())
    }
}

// ============================================================
// Which group of presets to show
// ============================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PresetContext {
    BtcNominal,
    StockShares,
    OptionContracts,
    ShowAll,
}

impl PresetContext {
    /// Derive from the current SelectedAssets
    pub fn from_assets(assets: &crate::dat::SelectedAssets) -> Self {
        // First check the FinAss if we have one selected
        if let Some(finass) = &assets.selected_finass {
            match finass {
                dsta::FinAss::OptDeets(_) => return PresetContext::OptionContracts,
                dsta::FinAss::StkDeets(stk) => {
                    if stk.ticker_name.contains("BTC") {
                        return PresetContext::BtcNominal;
                    }
                    return PresetContext::StockShares;
                }
                dsta::FinAss::BndDeets(_) => return PresetContext::StockShares,
            }
        }
        // Fall back to emerald type
        if let Some(et) = &assets.selected_emerald_type {
            let ticker = et.base_ticker();
            if ticker.contains("BTC") {
                return PresetContext::BtcNominal;
            }
        }
        PresetContext::ShowAll
    }

    pub fn presets(&self) -> &'static [SallyPresetOption] {
        match self {
            PresetContext::BtcNominal => &[
                SallyPresetOption::TinyBTC,
                SallyPresetOption::SmallBTC,
                SallyPresetOption::MediumBTC,
                SallyPresetOption::LargeBTC,
            ],
            PresetContext::StockShares => &[
                SallyPresetOption::OneShare,
                SallyPresetOption::TenShares,
                SallyPresetOption::HundredShares,
            ],
            PresetContext::OptionContracts => &[
                SallyPresetOption::OneContract,
                SallyPresetOption::TenContracts,
            ],
            PresetContext::ShowAll => &[
                SallyPresetOption::TinyBTC,
                SallyPresetOption::SmallBTC,
                SallyPresetOption::MediumBTC,
                SallyPresetOption::LargeBTC,
                SallyPresetOption::OneShare,
                SallyPresetOption::TenShares,
                SallyPresetOption::HundredShares,
                SallyPresetOption::OneContract,
                SallyPresetOption::TenContracts,
            ],
        }
    }

    pub fn header(&self) -> &'static str {
        match self {
            PresetContext::BtcNominal      => "BTC Nominal Presets",
            PresetContext::StockShares     => "Stock Share Presets",
            PresetContext::OptionContracts => "Option Contract Presets",
            PresetContext::ShowAll         => "All Presets",
        }
    }
}

// ============================================================
// SallyPresets state
// ============================================================

#[derive(Debug, Clone)]
pub struct SallyPresets {
    pub selected: Option<SallyPresetOption>,
}

#[derive(Debug, Clone)]
pub enum Message {
    SelectPreset(SallyPresetOption),
}

impl SallyPresets {
    pub fn new() -> Self {
        SallyPresets { selected: None }
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::SelectPreset(preset) => {
                self.selected = Some(preset);
            }
        }
    }
}

// ============================================================
// View
// ============================================================

pub fn view<'a>(
    presets: &'a SallyPresets,
    selected_assets: &'a crate::dat::SelectedAssets,
    on_select: impl Fn(SallyPresetOption) -> crate::Message + 'static + Copy,
) -> iced::Element<'a, crate::Message> {
    use iced::widget::{button, column, container, row, text, Space};
    use iced::{Alignment, Length};

    let ctx = PresetContext::from_assets(selected_assets);
    let preset_list = ctx.presets();

    let mut col = column![]
        .spacing(8)
        .padding(10)
        .width(Length::Fill);

    col = col.push(text(ctx.header()).size(16));
    col = col.push(Space::new().height(Length::Fixed(8.0)));

    // Show asset info if we have it
    if let Some(finass) = &selected_assets.selected_finass {
        let ask_str = finass.last_ticker()
            .map(|tk| format!("Current ask: ${:.4}", tk.ask))
            .unwrap_or_else(|| "No price data".to_string());
        col = col.push(
            text(format!("Asset: {}  {}", finass.order_name(), ask_str)).size(11)
        );
        col = col.push(Space::new().height(Length::Fixed(4.0)));
    } else if let Some(et) = &selected_assets.selected_emerald_type {
        col = col.push(text(format!("Asset: {}", et.label())).size(11));
        col = col.push(Space::new().height(Length::Fixed(4.0)));
    } else {
        col = col.push(
            text("← Select an asset from the emerald panel first").size(11)
        );
        col = col.push(Space::new().height(Length::Fixed(4.0)));
    }

    // Preset buttons
    for &preset in preset_list {
        let is_selected = presets.selected == Some(preset);
        let label = preset.label();

        let btn = button(
            container(text(label).size(13))
                .width(Length::Fill)
                .center_x(Length::Fill)
                .padding(8),
        )
        .width(Length::Fill)
        .on_press(on_select(preset));

        col = col.push(btn);
    }

    container(col)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_theme| iced::widget::container::Style {
            background: Some(iced::Background::Color(
                iced::Color::from_rgba(0.05, 0.19, 0.49, 0.15),
            )),
            ..Default::default()
        })
        .into()
}