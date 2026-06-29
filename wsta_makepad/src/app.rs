// trade/wsta_makepad/src/app.rs
//
// Makepad/WASM replacement for vsta.
// First final screen: Dr. Robotnik make-bot constructor hub.

use makepad_widgets::*;
use makepad_widgets::{app_main, Live, LiveHook};

use crate::protocol::*;
use crate::transport::WstaTransport;

#[rustfmt::skip]
live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    WstaButton = <Button> {
        width: Fill,
        height: 74,
        draw_text: {
            color: #dce7ff
            text_style: {font_size: 11.0}
        }
    }

    WstaToolButton = <Button> {
        width: Fill,
        height: 92,
        draw_text: {
            color: #dce7ff
            text_style: {font_size: 10.0}
        }
    }

    WstaInput = <TextInput> {
        width: Fill,
        height: Fit,
        draw_text: {
            color: #ffffff
            text_style: {font_size: 12.0}
        }
    }

    WstaPanel = <View> {
        width: Fill,
        height: Fit,
        flow: Down,
        spacing: 8,
        padding: {left: 12, top: 12, right: 12, bottom: 12}
        show_bg: true
        draw_bg: {color: #07101f88}
    }

    App = {{App}} {
        ui: <Root> {
            main_window = <Window> {
                window: {inner_size: vec2(1220, 760)}
                body = <View> {
                    flow: Down,
                    width: Fill,
                    height: Fill,
                    show_bg: true
                    draw_bg: {color: #05070b}

                    main_area = <View> {
                        flow: Right,
                        width: Fill,
                        height: Fill

                        nav = <View> {
                            flow: Down,
                            width: 240,
                            height: Fill,
                            spacing: 8,
                            padding: {left: 12, top: 12, right: 12, bottom: 12}
                            show_bg: true
                            draw_bg: {color: #07101fee}

                            title = <Label> {
                                text: "WSTA"
                                draw_text: {
                                    color: #70d6ff
                                    text_style: {font_size: 26.0}
                                }
                            }

                            dr_nav = <WstaButton> {text: "Dr. Robotnik"}
                            buzz_nav = <WstaButton> {text: "Buzz"}
                            stealth_nav = <WstaButton> {text: "Stealth"}
                            sally_nav = <WstaButton> {text: "Sally"}
                            swat_nav = <WstaButton> {text: "Swat"}
                            ttai_nav = <WstaButton> {text: "TTAI"}
                            nico_nav = <WstaButton> {text: "Nico"}
                            logs_nav = <WstaButton> {text: "Logs"}

                            transport_label = <Label> {
                                text: "transport: booting"
                                draw_text: {
                                    color: #92a3c4
                                    text_style: {font_size: 11.0}
                                }
                            }
                        }

                        content = <View> {
                            flow: Down,
                            width: Fill,
                            height: Fill

                            header = <View> {
                                flow: Down,
                                width: Fill,
                                height: 74,
                                padding: {left: 14, top: 10, right: 14, bottom: 8}
                                show_bg: true
                                draw_bg: {color: #060a16dd}

                                view_title = <Label> {
                                    text: "Dr. Robotnik"
                                    draw_text: {
                                        color: #70d6ff
                                        text_style: {font_size: 24.0}
                                    }
                                }

                                view_subtitle = <Label> {
                                    text: "bot constructor and backend overview"
                                    draw_text: {
                                        color: #92a3c4
                                        text_style: {font_size: 12.0}
                                    }
                                }
                            }

                            dr_surface = <View> {
                                flow: Right,
                                width: Fill,
                                height: Fill,
                                spacing: 12,
                                padding: {left: 12, top: 12, right: 12, bottom: 12}

                                dr_tools = <View> {
                                    flow: Down,
                                    width: 132,
                                    height: Fill,
                                    spacing: 8,
                                    padding: {left: 8, top: 8, right: 8, bottom: 8}
                                    show_bg: true
                                    draw_bg: {color: #07101faa}

                                    dr_overview_tool = <WstaToolButton> {text: "Overview"}
                                    dr_buzz_tool = <WstaToolButton> {text: "Make Buzz"}
                                    dr_stealth_tool = <WstaToolButton> {text: "Make Stealth"}
                                    dr_sally_tool = <WstaToolButton> {text: "Make Sally"}
                                    dr_swat_tool = <WstaToolButton> {text: "Make Swat"}
                                    dr_ttai_tool = <WstaToolButton> {text: "TTAI"}
                                }

                                dr_display = <View> {
                                    flow: Down,
                                    width: Fill,
                                    height: Fill,
                                    spacing: 10,
                                    padding: {left: 14, top: 14, right: 14, bottom: 14}
                                    show_bg: true
                                    draw_bg: {color: #05070baa}

                                    display_title = <Label> {
                                        text: "Overview"
                                        draw_text: {
                                            color: #70d6ff
                                            text_style: {font_size: 20.0}
                                        }
                                    }

                                    form_grid = <View> {
                                        flow: Down,
                                        width: Fill,
                                        height: Fit,
                                        spacing: 8

                                        common_panel = <WstaPanel> {
                                            common_title = <Label> {text: "Common Bot Info"}
                                            friendly_name = <WstaInput> {text: "Buzz Bot 1"}
                                            tracking_tick = <WstaInput> {text: "SPY"}
                                            max_risk_percent = <WstaInput> {text: "0"}
                                            max_risk_dollar = <WstaInput> {text: "0"}
                                        }

                                        maker_panel = <WstaPanel> {
                                            maker_title = <Label> {text: "Maker Fields"}
                                            cash_alloc = <WstaInput> {text: "150"}
                                            market_direction = <WstaInput> {text: "GetStonk"}
                                            option_expire = <WstaInput> {text: "5"}
                                            target_spread = <WstaInput> {text: "0.25"}
                                            option_bucket = <WstaInput> {text: "0"}
                                            spread_bucket = <WstaInput> {text: "1"}
                                            exit_gain_pct = <WstaInput> {text: "50"}
                                            exit_loss_pct = <WstaInput> {text: "-50"}
                                            cooldown_seconds = <WstaInput> {text: "0"}
                                            order_ticker = <WstaInput> {text: "BTC/USD"}
                                            order_quantity = <WstaInput> {text: "1"}
                                            order_price = <WstaInput> {text: "42"}
                                            reveal_price = <WstaInput> {text: "42"}
                                        }

                                        action_panel = <WstaPanel> {
                                            action_title = <Label> {text: "Timing / Exit"}
                                            entry_relative_days = <WstaInput> {text: "0"}
                                            entry_market_time = <WstaInput> {text: "PowerEnds"}
                                            entry_wait_seconds = <WstaInput> {text: "0"}
                                            exit_kind = <WstaInput> {text: "SpreadValueGain"}
                                            exit_value = <WstaInput> {text: "0.05"}
                                        }

                                        submit_row = <View> {
                                            flow: Right,
                                            width: Fill,
                                            height: Fit,
                                            spacing: 8

                                            create_bot = <Button> {text: "Create Selected Bot"}
                                            report_status = <Button> {text: "Report Status"}
                                            send_log = <Button> {text: "Send Log"}
                                        }
                                    }
                                }
                            }
                        }
                    }

                    debug = <View> {
                        flow: Down,
                        width: Fill,
                        height: 170,
                        padding: {left: 10, top: 8, right: 10, bottom: 8}
                        show_bg: true
                        draw_bg: {color: #040914f4}

                        debug_title = <Label> {
                            text: "debug / backend info"
                            draw_text: {
                                color: #70d6ff
                                text_style: {font_size: 12.0}
                            }
                        }

                        debug_text = <Label> {
                            text: "wsta_makepad booting"
                            draw_text: {
                                color: #aac8f0
                                text_style: {font_size: 11.0}
                            }
                        }
                    }
                }
            }
        }
    }
}

app_main!(App);

#[derive(Live, LiveHook)]
pub struct App {
    #[live]
    ui: WidgetRef,

    #[rust]
    selected_view: WstaView,

    #[rust]
    selected_dr_tool: DrTool,

    #[rust]
    transport: WstaTransport,

    #[rust]
    debug_lines: Vec<String>,
}

impl LiveRegister for App {
    fn live_register(cx: &mut Cx) {
        makepad_widgets::live_design(cx);
    }
}

impl MatchEvent for App {
    fn handle_startup(&mut self, cx: &mut Cx) {
        self.selected_view = WstaView::DrR;
        self.selected_dr_tool = DrTool::Overview;
        self.transport = WstaTransport::new();
        self.debug_lines = vec!["wsta_makepad started".to_string()];
        self.sync_ui(cx);
        self.send_packet(cx, BrowserToWsta::SelectView { view: WstaView::DrR });
    }

    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions) {
        if self.ui.button(id!(dr_nav)).clicked(actions) {
            self.selected_view = WstaView::DrR;
            self.send_packet(cx, BrowserToWsta::SelectView { view: WstaView::DrR });
            self.sync_ui(cx);
            return;
        }

        if self.ui.button(id!(buzz_nav)).clicked(actions) {
            self.selected_view = WstaView::Buzz;
            self.send_packet(cx, BrowserToWsta::SelectView { view: WstaView::Buzz });
            self.sync_ui(cx);
            return;
        }

        if self.ui.button(id!(stealth_nav)).clicked(actions) {
            self.selected_view = WstaView::Stealth;
            self.send_packet(cx, BrowserToWsta::SelectView { view: WstaView::Stealth });
            self.sync_ui(cx);
            return;
        }

        if self.ui.button(id!(sally_nav)).clicked(actions) {
            self.selected_view = WstaView::Sally;
            self.send_packet(cx, BrowserToWsta::SelectView { view: WstaView::Sally });
            self.sync_ui(cx);
            return;
        }

        if self.ui.button(id!(swat_nav)).clicked(actions) {
            self.selected_view = WstaView::Swat;
            self.send_packet(cx, BrowserToWsta::SelectView { view: WstaView::Swat });
            self.sync_ui(cx);
            return;
        }

        if self.ui.button(id!(ttai_nav)).clicked(actions) {
            self.selected_view = WstaView::Ttai;
            self.send_packet(cx, BrowserToWsta::SelectView { view: WstaView::Ttai });
            self.sync_ui(cx);
            return;
        }

        if self.ui.button(id!(nico_nav)).clicked(actions) {
            self.selected_view = WstaView::Nico;
            self.send_packet(cx, BrowserToWsta::SelectView { view: WstaView::Nico });
            self.sync_ui(cx);
            return;
        }

        if self.ui.button(id!(logs_nav)).clicked(actions) {
            self.selected_view = WstaView::Logs;
            self.send_packet(cx, BrowserToWsta::SelectView { view: WstaView::Logs });
            self.sync_ui(cx);
            return;
        }

        if self.ui.button(id!(dr_overview_tool)).clicked(actions) {
            self.selected_dr_tool = DrTool::Overview;
            self.sync_ui(cx);
            return;
        }

        if self.ui.button(id!(dr_buzz_tool)).clicked(actions) {
            self.selected_dr_tool = DrTool::MakeBuzz;
            self.set_default_names(cx, "Buzz Bot 1", "SPY");
            self.sync_ui(cx);
            return;
        }

        if self.ui.button(id!(dr_stealth_tool)).clicked(actions) {
            self.selected_dr_tool = DrTool::MakeStealth;
            self.set_default_names(cx, "Stealth Bot 1", "SPY");
            self.sync_ui(cx);
            return;
        }

        if self.ui.button(id!(dr_sally_tool)).clicked(actions) {
            self.selected_dr_tool = DrTool::MakeSally;
            self.set_default_names(cx, "Sally Fakes 1", "BTC/USD");
            self.sync_ui(cx);
            return;
        }

        if self.ui.button(id!(dr_swat_tool)).clicked(actions) {
            self.selected_dr_tool = DrTool::MakeSwat;
            self.set_default_names(cx, "Swat Bot 1", "SPY");
            self.sync_ui(cx);
            return;
        }

        if self.ui.button(id!(dr_ttai_tool)).clicked(actions) {
            self.selected_dr_tool = DrTool::TtaiOverview;
            self.sync_ui(cx);
            return;
        }

        if self.ui.button(id!(create_bot)).clicked(actions) {
            if let Some(pkt) = self.build_selected_create_packet() {
                self.send_packet(cx, pkt);
            }
            return;
        }

        if self.ui.button(id!(report_status)).clicked(actions) {
            self.send_packet(cx, BrowserToWsta::ReportOfAllStatus);
            return;
        }

        if self.ui.button(id!(send_log)).clicked(actions) {
            self.send_packet(cx, BrowserToWsta::SendLogNote {
                text: "hello from wsta_makepad".to_string(),
            });
        }
    }
}

impl AppMain for App {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event) {
        self.match_event(cx, event);
        self.ui.handle_event(cx, event, &mut Scope::empty());
    }
}

impl App {
    fn sync_ui(&mut self, cx: &mut Cx) {
        let (title, subtitle) = match self.selected_view {
            WstaView::DrR => ("Dr. Robotnik", "bot constructor and backend overview"),
            WstaView::Buzz => ("Buzz", "Buzz Bomber live view"),
            WstaView::Stealth => ("Stealth", "Stealth bot live view"),
            WstaView::Sally => ("Sally", "Sally fake-order live view"),
            WstaView::Swat => ("Swat", "SWAT bot live view"),
            WstaView::Ttai => ("TTAI", "account / positions / order updates"),
            WstaView::Nico => ("Nico", "assistant/chat control"),
            WstaView::Logs => ("Logs", "backend and frontend log notes"),
        };

        self.ui.label(id!(view_title)).set_text(cx, title);
        self.ui.label(id!(view_subtitle)).set_text(cx, subtitle);

        let display = match self.selected_dr_tool {
            DrTool::Overview => "Overview",
            DrTool::MakeBuzz => "Make Buzz Bomber",
            DrTool::MakeStealth => "Make Stealth Bot",
            DrTool::MakeSally => "Make Sally Fakes",
            DrTool::MakeSwat => "Make Swat Bot",
            DrTool::TtaiOverview => "TTAI Overview",
        };

        self.ui.label(id!(display_title)).set_text(cx, display);
        self.ui
            .label(id!(transport_label))
            .set_text(cx, "transport: WebTransport adapter");
        self.ui
            .label(id!(debug_text))
            .set_text(cx, &self.debug_lines.join("\n"));
    }

    fn set_default_names(&mut self, cx: &mut Cx, name: &str, ticker: &str) {
        self.ui.text_input(id!(friendly_name)).set_text(cx, name);
        self.ui.text_input(id!(tracking_tick)).set_text(cx, ticker);
    }

    fn send_packet(&mut self, cx: &mut Cx, pkt: BrowserToWsta) {
        match self.transport.send(&pkt) {
            Ok(()) => self.debug_lines.insert(0, format!("SEND {:?}", pkt)),
            Err(e) => self.debug_lines.insert(0, format!("SEND ERROR {}", e)),
        }

        self.debug_lines.truncate(14);
        self.sync_ui(cx);
    }

    fn text(&self, id: &[LiveId]) -> String {
        self.ui.text_input(id).text()
    }

    fn f64_text(&self, id: &[LiveId]) -> f64 {
        self.text(id).parse::<f64>().unwrap_or(0.0)
    }

    fn u16_text(&self, id: &[LiveId]) -> u16 {
        self.text(id).parse::<u16>().unwrap_or(0)
    }

    fn u8_text(&self, id: &[LiveId]) -> u8 {
        self.text(id).parse::<u8>().unwrap_or(0)
    }

    fn i64_text(&self, id: &[LiveId]) -> i64 {
        self.text(id).parse::<i64>().unwrap_or(0)
    }

    fn common(&self) -> CommonBotInfoInput {
        CommonBotInfoInput {
            friendly_name: self.text(id!(friendly_name)),
            tracking_tick: self.text(id!(tracking_tick)),
            max_cash_risk_use_max: false,
            max_cash_risk_percent: self.f64_text(id!(max_risk_percent)),
            max_cash_risk_dollar: self.f64_text(id!(max_risk_dollar)),
        }
    }

    fn entry_time(&self) -> BotActionTimeInput {
        BotActionTimeInput {
            relative_days: self.u16_text(id!(entry_relative_days)),
            market_time: self.text(id!(entry_market_time)),
            wait_seconds: self.i64_text(id!(entry_wait_seconds)),
        }
    }

    fn build_selected_create_packet(&self) -> Option<BrowserToWsta> {
        match self.selected_dr_tool {
            DrTool::MakeBuzz => Some(BrowserToWsta::CreateBuzzBot(CreateBuzzBotInput {
                common: self.common(),
                cash_alloc: self.f64_text(id!(cash_alloc)),
                market_direction: self.text(id!(market_direction)),
                option_expire: self.u16_text(id!(option_expire)),
                target_spread: self.f64_text(id!(target_spread)),
                time_to_order: self.entry_time(),
                follow_a_exit: vec![BuzzExitInput {
                    exit_kind: self.text(id!(exit_kind)),
                    value: self.f64_text(id!(exit_value)),
                    time: Some(self.entry_time()),
                }],
                algo_cooldown_seconds: self.i64_text(id!(cooldown_seconds)),
                bombs_forever: true,
            })),

            DrTool::MakeStealth => Some(BrowserToWsta::CreateStealthBot(CreateStealthBotInput {
                common: self.common(),
                cash_alloc: self.f64_text(id!(cash_alloc)),
                market_direction: self.text(id!(market_direction)),
                option_expire: self.u16_text(id!(option_expire)),
                option_bucket: self.u8_text(id!(option_bucket)),
                spread_bucket: self.u8_text(id!(spread_bucket)),
                exit_gain_pct: self.f64_text(id!(exit_gain_pct)),
                exit_loss_pct: self.f64_text(id!(exit_loss_pct)),
                nice_exit_way: vec![StealthExitInput {
                    exit_kind: self.text(id!(exit_kind)),
                    value: self.f64_text(id!(exit_value)),
                    time: Some(self.entry_time()),
                }],
                use_theo_cost: false,
            })),

            DrTool::MakeSally => Some(BrowserToWsta::CreateSallyBot(CreateSallyBotInput {
                common: self.common(),
                order: SallyOrderInput {
                    ticker: self.text(id!(order_ticker)),
                    quantity: self.f64_text(id!(order_quantity)),
                    price: self.f64_text(id!(order_price)),
                    action: "BuyToOpen".to_string(),
                    order_type: "Limit".to_string(),
                },
                reveal: SallyRevealInput {
                    reveal_kind: "SubmitRightAwayButLikeOnlyUseForTest".to_string(),
                    price: self.f64_text(id!(reveal_price)),
                    time: Some(self.entry_time()),
                },
            })),

            DrTool::MakeSwat => Some(BrowserToWsta::CreateSwatBot(CreateSwatBotInput {
                common: self.common(),
            })),

            DrTool::Overview | DrTool::TtaiOverview => None,
        }
    }
}
