// trade/wsta_makepad/src/app.rs
//
// Final Makepad/WASM GUI elements for WSTA.
// This intentionally avoids per-field live mutation tricks for this pass: the
// visible controls are Makepad widgets, while selected forms emit canonical
// BrowserToWsta packets with sane defaults.
//
// Image-backed controls:
// Use resources/images/*.jpg through Makepad resource paths. On web these are
// deployed as server-hosted files inside the generated package/resource tree;
// they should not be embedded into the wasm binary. The next visual pass can
// replace plain WstaButton/WstaToolButton draw backgrounds with image-backed
// draw/image widgets while keeping this same layout skeleton.
//
// Next patches can wire every TextInput field into packet construction one
// field at a time.

use makepad_widgets::*;

use crate::protocol::*;
use crate::transport::WstaTransport;

#[rustfmt::skip]
live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    WstaButton = <Button> {
        width: Fill,
        height: 52,
        margin: {bottom: 6}
        draw_text: {text_style: {font_size: 11.0}}
    }

    WstaToolButton = <Button> {
        width: Fill,
        height: 62,
        margin: {bottom: 6}
        draw_text: {text_style: {font_size: 10.0}}
    }

    WstaInput = <TextInput> {
        width: Fill,
        height: 34,
        margin: {bottom: 6}
    }

    WstaLabel = <Label> {
        width: Fill,
        height: Fit,
        draw_text: {color: #dce7ff, text_style: {font_size: 11.0}}
    }

    WstaPanel = <View> {
        width: Fill,
        height: Fit,
        flow: Down,
        spacing: 5,
        padding: {left: 12, top: 12, right: 12, bottom: 12}
        margin: {bottom: 10}
        show_bg: true,
        draw_bg: {color: #07101f88}
    }

    App = {{App}} {
        ui: <Root> {
            main_window = <Window> {
                window: {inner_size: vec2(1240, 780)}
                body = <View> {
                    width: Fill,
                    height: Fill,
                    flow: Down,
                    show_bg: true,
                    draw_bg: {color: #05070b}

                    top = <View> {
                        width: Fill,
                        height: Fill,
                        flow: Right,

                        nav = <View> {
                            width: 240,
                            height: Fill,
                            flow: Down,
                            padding: {left: 12, top: 12, right: 12, bottom: 12}
                            show_bg: true,
                            draw_bg: {color: #07101fee}

                            title = <Label> {text: "WSTA", draw_text: {color: #70d6ff, text_style: {font_size: 28.0}}}
                            subtitle = <Label> {text: "Makepad / WASM", draw_text: {color: #92a3c4, text_style: {font_size: 11.0}}}
                            dr_nav = <WstaButton> {text: "Dr. Robotnik"}
                            buzz_nav = <WstaButton> {text: "Buzz"}
                            stealth_nav = <WstaButton> {text: "Stealth"}
                            sally_nav = <WstaButton> {text: "Sally"}
                            swat_nav = <WstaButton> {text: "Swat"}
                            ttai_nav = <WstaButton> {text: "TTAI"}
                            nico_nav = <WstaButton> {text: "Nico"}
                            logs_nav = <WstaButton> {text: "Logs"}
                            transport_label = <Label> {text: "transport: UI loaded / backend retrying", draw_text: {color: #ffd166, text_style: {font_size: 11.0}}}
                        }

                        content = <View> {
                            width: Fill,
                            height: Fill,
                            flow: Down,

                            header = <View> {
                                width: Fill,
                                height: 74,
                                flow: Down,
                                padding: {left: 14, top: 9, right: 14, bottom: 8}
                                show_bg: true,
                                draw_bg: {color: #060a16dd}
                                view_title = <Label> {text: "Dr. Robotnik", draw_text: {color: #70d6ff, text_style: {font_size: 24.0}}}
                                view_subtitle = <Label> {text: "bot constructor and backend overview", draw_text: {color: #92a3c4, text_style: {font_size: 12.0}}}
                            }

                            dr_body = <View> {
                                width: Fill,
                                height: Fill,
                                flow: Right,
                                spacing: 12,
                                padding: {left: 12, top: 12, right: 12, bottom: 12}

                                dr_tools = <View> {
                                    width: 132,
                                    height: Fill,
                                    flow: Down,
                                    padding: {left: 8, top: 8, right: 8, bottom: 8}
                                    show_bg: true,
                                    draw_bg: {color: #07101faa}
                                    dr_overview_tool = <WstaToolButton> {text: "Overview"}
                                    dr_buzz_tool = <WstaToolButton> {text: "Make Buzz"}
                                    dr_stealth_tool = <WstaToolButton> {text: "Make Stealth"}
                                    dr_sally_tool = <WstaToolButton> {text: "Make Sally"}
                                    dr_swat_tool = <WstaToolButton> {text: "Make Swat"}
                                    dr_ttai_tool = <WstaToolButton> {text: "TTAI"}
                                }

                                display = <ScrollYView> {
                                    width: Fill,
                                    height: Fill,
                                    flow: Down,
                                    padding: {left: 14, top: 14, right: 14, bottom: 14}
                                    show_bg: true,
                                    draw_bg: {color: #05070baa}

                                    display_title = <Label> {text: "Overview", draw_text: {color: #70d6ff, text_style: {font_size: 20.0}}}
                                    display_hint = <Label> {text: "Select a MakeBot option on the left.", draw_text: {color: #aac8f0, text_style: {font_size: 12.0}}}

                                    common_panel = <WstaPanel> {
                                        common_title = <Label> {text: "Common Bot Info"}
                                        friendly_name = <WstaInput> {text: "Buzz Bot 1"}
                                        tracking_tick = <WstaInput> {text: "SPY"}
                                        max_risk_percent = <WstaInput> {text: "0"}
                                        max_risk_dollar = <WstaInput> {text: "0"}
                                    }

                                    buzz_panel = <WstaPanel> {
                                        buzz_title = <Label> {text: "Buzz / Stealth Fields"}
                                        cash_alloc = <WstaInput> {text: "150"}
                                        market_direction = <WstaInput> {text: "GetStonk"}
                                        option_expire = <WstaInput> {text: "5"}
                                        target_spread = <WstaInput> {text: "0.25"}
                                        option_bucket = <WstaInput> {text: "0"}
                                        spread_bucket = <WstaInput> {text: "1"}
                                        exit_gain_pct = <WstaInput> {text: "50"}
                                        exit_loss_pct = <WstaInput> {text: "-50"}
                                    }

                                    timing_panel = <WstaPanel> {
                                        timing_title = <Label> {text: "Timing / Exit"}
                                        entry_relative_days = <WstaInput> {text: "0"}
                                        entry_market_time = <WstaInput> {text: "PowerEnds"}
                                        entry_wait_seconds = <WstaInput> {text: "0"}
                                        exit_kind = <WstaInput> {text: "SpreadValueGain"}
                                        exit_value = <WstaInput> {text: "0.05"}
                                    }

                                    sally_panel = <WstaPanel> {
                                        sally_title = <Label> {text: "Sally Hidden Order / Reveal"}
                                        order_ticker = <WstaInput> {text: "BTC/USD"}
                                        order_quantity = <WstaInput> {text: "1"}
                                        order_price = <WstaInput> {text: "42"}
                                        reveal_price = <WstaInput> {text: "42"}
                                    }

                                    actions = <View> {
                                        width: Fill,
                                        height: Fit,
                                        flow: Right,
                                        spacing: 8,
                                        create_bot = <Button> {text: "Create Selected Bot"}
                                        report_status = <Button> {text: "Report Status"}
                                        send_log = <Button> {text: "Send Log"}
                                    }
                                }
                            }
                        }
                    }

                    debug = <View> {
                        width: Fill,
                        height: 170,
                        flow: Down,
                        padding: {left: 10, top: 8, right: 10, bottom: 8}
                        show_bg: true,
                        draw_bg: {color: #040914f4}
                        debug_title = <Label> {text: "debug / backend info", draw_text: {color: #70d6ff, text_style: {font_size: 12.0}}}
                        debug_text = <Label> {text: "wsta_makepad UI booting; backend connection is optional", draw_text: {color: #aac8f0, text_style: {font_size: 11.0}}}
                    }
                }
            }
        }
    }
}

app_main!(App);

#[derive(Live, LiveHook)]
pub struct App {
    #[live] ui: WidgetRef,
    #[rust] selected_view: WstaView,
    #[rust] selected_dr_tool: DrTool,
    #[rust] transport: WstaTransport,
    #[rust] debug_lines: Vec<String>,
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
        self.debug_lines = vec!["wsta_makepad UI started; backend transport may still be retrying".to_string()];

        self.sync_ui(cx);
        self.send_packet(cx, BrowserToWsta::SelectView { view: WstaView::DrR });
    }

    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions) {
        if self.ui.button(id!(dr_nav)).clicked(actions) {
            self.select_view(cx, WstaView::DrR);
        }

        if self.ui.button(id!(buzz_nav)).clicked(actions) {
            self.select_view(cx, WstaView::Buzz);
        }

        if self.ui.button(id!(stealth_nav)).clicked(actions) {
            self.select_view(cx, WstaView::Stealth);
        }

        if self.ui.button(id!(sally_nav)).clicked(actions) {
            self.select_view(cx, WstaView::Sally);
        }

        if self.ui.button(id!(swat_nav)).clicked(actions) {
            self.select_view(cx, WstaView::Swat);
        }

        if self.ui.button(id!(ttai_nav)).clicked(actions) {
            self.select_view(cx, WstaView::Ttai);
        }

        if self.ui.button(id!(nico_nav)).clicked(actions) {
            self.select_view(cx, WstaView::Nico);
        }

        if self.ui.button(id!(logs_nav)).clicked(actions) {
            self.select_view(cx, WstaView::Logs);
        }

        if self.ui.button(id!(dr_overview_tool)).clicked(actions) {
            self.select_tool(cx, DrTool::Overview);
        }

        if self.ui.button(id!(dr_buzz_tool)).clicked(actions) {
            self.select_tool(cx, DrTool::MakeBuzz);
        }

        if self.ui.button(id!(dr_stealth_tool)).clicked(actions) {
            self.select_tool(cx, DrTool::MakeStealth);
        }

        if self.ui.button(id!(dr_sally_tool)).clicked(actions) {
            self.select_tool(cx, DrTool::MakeSally);
        }

        if self.ui.button(id!(dr_swat_tool)).clicked(actions) {
            self.select_tool(cx, DrTool::MakeSwat);
        }

        if self.ui.button(id!(dr_ttai_tool)).clicked(actions) {
            self.select_tool(cx, DrTool::TtaiOverview);
        }

        if self.ui.button(id!(create_bot)).clicked(actions) {
            if let Some(pkt) = self.make_packet() {
                self.send_packet(cx, pkt);
            }
        }

        if self.ui.button(id!(report_status)).clicked(actions) {
            self.send_packet(cx, BrowserToWsta::ReportOfAllStatus);
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
    fn select_view(&mut self, cx: &mut Cx, view: WstaView) {
        self.selected_view = view;
        self.send_packet(cx, BrowserToWsta::SelectView { view });
        self.sync_ui(cx);
    }

    fn select_tool(&mut self, cx: &mut Cx, tool: DrTool) {
        self.selected_dr_tool = tool;
        self.sync_ui(cx);
    }

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
        self.ui.label(id!(display_hint)).set_text(cx, self.form_hint());
        self.ui.label(id!(transport_label)).set_text(cx, &self.transport.status_line());
        self.ui.label(id!(debug_text)).set_text(cx, &self.debug_lines.join("\n"));
    }

    fn form_hint(&self) -> &'static str {
        match self.selected_dr_tool {
            DrTool::Overview => "Select a MakeBot option on the left.",
            DrTool::MakeBuzz => "Buzz uses Common + Buzz fields + Timing/Exit.",
            DrTool::MakeStealth => "Stealth uses Common + Buzz/Stealth fields + Timing/Exit.",
            DrTool::MakeSally => "Sally uses Common + Hidden Order / Reveal.",
            DrTool::MakeSwat => "Swat currently sends CommonBotInfo only because dsta::MakeSwatBotsGo only has my_accounting.",
            DrTool::TtaiOverview => "Use Report Status to request account/positions/orders.",
        }
    }

    fn send_packet(&mut self, cx: &mut Cx, pkt: BrowserToWsta) {
        match self.transport.send(&pkt) {
            Ok(()) => self.debug_lines.insert(0, format!("SEND {:?}", pkt)),
            Err(e) => self.debug_lines.insert(0, format!("SEND ERROR {}", e)),
        }
        self.debug_lines.truncate(12);
        self.sync_ui(cx);
    }

    fn make_packet(&self) -> Option<BrowserToWsta> {
        match self.selected_dr_tool {
            DrTool::MakeBuzz => Some(BrowserToWsta::CreateBuzzBot(CreateBuzzBotInput {
                common: common("Buzz Bot 1", "SPY"),
                cash_alloc: 150.0,
                market_direction: "GetStonk".to_string(),
                option_expire: 5,
                target_spread: 0.25,
                time_to_order: action_time(),
                follow_a_exit: vec![BuzzExitInput { exit_kind: "SpreadValueGain".to_string(), value: 0.05, time: Some(action_time()) }],
                algo_cooldown_seconds: 0,
                bombs_forever: true,
            })),
            DrTool::MakeStealth => Some(BrowserToWsta::CreateStealthBot(CreateStealthBotInput {
                common: common("Stealth Bot 1", "SPY"),
                cash_alloc: 150.0,
                market_direction: "GetStonk".to_string(),
                option_expire: 5,
                option_bucket: 0,
                spread_bucket: 1,
                exit_gain_pct: 50.0,
                exit_loss_pct: -50.0,
                nice_exit_way: vec![StealthExitInput { exit_kind: "OtmShort".to_string(), value: 0.0, time: Some(action_time()) }],
                use_theo_cost: false,
            })),
            DrTool::MakeSally => Some(BrowserToWsta::CreateSallyBot(CreateSallyBotInput {
                common: common("Sally Fakes 1", "BTC/USD"),
                order: SallyOrderInput { ticker: "BTC/USD".to_string(), quantity: 1.0, price: 42.0, action: "BuyToOpen".to_string(), order_type: "Limit".to_string() },
                reveal: SallyRevealInput { reveal_kind: "SubmitRightAwayButLikeOnlyUseForTest".to_string(), price: 42.0, time: Some(action_time()) },
            })),
            DrTool::MakeSwat => Some(BrowserToWsta::CreateSwatBot(CreateSwatBotInput { common: common("Swat Bot 1", "SPY") })),
            DrTool::Overview | DrTool::TtaiOverview => None,
        }
    }
}
