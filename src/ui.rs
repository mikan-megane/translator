use crate::{cfg::get_theme, font};
use egui::{self, epaint::Color32};
use std::{
    io::Cursor,
    sync::{mpsc, Arc, Mutex},
};

#[cfg(target_os = "windows")]
use crate::hotkey::HotkeySetting;
#[cfg(target_os = "windows")]
use std::sync::mpsc::Receiver;

pub const LINK_COLOR_DOING: Color32 = Color32::GREEN;
pub const LINK_COLOR_COMMON: Color32 = Color32::GRAY;

pub struct State {
    pub text: String,
    pub source_lang: deepl::Lang,
    pub target_lang: deepl::Lang,
    pub link_color: Color32,
}

pub struct MyApp {
    state: Arc<Mutex<State>>,

    lang_list_with_auto: Vec<deepl::Lang>,
    lang_list: Vec<deepl::Lang>,
    task_chan: mpsc::SyncSender<()>,

    #[cfg(target_os = "windows")]
    hk_setting: HotkeySetting,
    #[cfg(target_os = "windows")]
    hotkey_rx: Receiver<()>,
}

pub fn description(lang: deepl::Lang) -> &'static str {
    match lang {
        deepl::Lang::Auto => "自動検出",
        deepl::Lang::DE => "ドイツ語",
        deepl::Lang::EN => "英語",
        deepl::Lang::ES => "スペイン語",
        deepl::Lang::FR => "フランス語",
        deepl::Lang::IT => "イタリア語",
        deepl::Lang::JA => "日本語",
        deepl::Lang::NL => "オランダ語",
        deepl::Lang::PL => "ポーランド語",
        deepl::Lang::PT => "ポルトガル語",
        deepl::Lang::RU => "ロシア語",
        deepl::Lang::ZH => "中国語",
        deepl::Lang::BG => "ブルガリア語",
        deepl::Lang::CS => "チェコ語",
        deepl::Lang::DA => "デンマーク語",
        deepl::Lang::EL => "ギリシャ語",
        deepl::Lang::ET => "エストニア語",
        deepl::Lang::FI => "フィンランド語",
        deepl::Lang::HU => "ハンガリー語",
        deepl::Lang::LT => "リトアニア語",
        deepl::Lang::LV => "ラトビア語",
        deepl::Lang::RO => "ルーマニア語",
        deepl::Lang::SK => "スロバキア語",
        deepl::Lang::SL => "スロベニア語",
        deepl::Lang::SV => "スウェーデン語",
    }
}

impl MyApp {
    pub fn new(
        state: Arc<Mutex<State>>,

        task_chan: mpsc::SyncSender<()>,
        cc: &eframe::CreationContext<'_>,
    ) -> Self {
        font::install_fonts(&cc.egui_ctx);

        match get_theme().as_str() {
            "light" => cc.egui_ctx.set_visuals(egui::Visuals::light()),
            _ => cc.egui_ctx.set_visuals(egui::Visuals::dark()),
        }

        #[cfg(target_os = "windows")]
        let (hotkey_tx, hotkey_rx) = mpsc::sync_channel(1);
        #[cfg(target_os = "windows")]
        let mut hk_setting = HotkeySetting::default();
        #[cfg(target_os = "windows")]
        hk_setting.register_hotkey(hotkey_tx);

        Self {
            state,

            lang_list_with_auto: deepl::Lang::lang_list_with_auto(),
            lang_list: deepl::Lang::lang_list(),
            task_chan,

            #[cfg(target_os = "windows")]
            hk_setting,
            #[cfg(target_os = "windows")]
            hotkey_rx,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let Self {
            state,

            lang_list_with_auto,
            lang_list,
            task_chan,

            #[cfg(target_os = "windows")]
            hk_setting,
            #[cfg(target_os = "windows")]
            hotkey_rx,
        } = self;
        frame.set_decorations(true);
        let mut state = state.lock().unwrap();

        let old_source_lang = state.source_lang;
        let old_target_lang = state.target_lang;

        if ctx.input().key_pressed(egui::Key::Escape) {
            #[cfg(target_os = "windows")]
            hk_setting.unregister_all();
            frame.close()
        }

        #[cfg(target_os = "windows")]
        if let Ok(_) = hotkey_rx.try_recv() {
            _ = task_chan.send(());
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.horizontal_top(|ui| {
                    let combobox_width = 145.0;
                    egui::ComboBox::from_id_source(egui::Id::new("source_lang_ComboBox"))
                        .selected_text(description(state.source_lang))
                        .width(combobox_width)
                        .show_ui(ui, |ui| {
                            for i in lang_list_with_auto {
                                let i = i.to_owned();
                                ui.selectable_value(&mut state.source_lang, i, description(i));
                            }
                        });

                    if ui.add(egui::Button::new(" ⇌ ").frame(false)).clicked() {
                        let tmp_target_lang = state.target_lang;
                        let tmp_source_lang = state.source_lang;
                        state.target_lang = if tmp_source_lang == deepl::Lang::Auto {
                            deepl::Lang::EN
                        } else {
                            tmp_source_lang
                        };
                        state.source_lang = tmp_target_lang;
                    };

                    egui::ComboBox::from_id_source(egui::Id::new("target_lang_ComboBox"))
                        .selected_text(description(state.target_lang))
                        .width(combobox_width)
                        .show_ui(ui, |ui| {
                            for i in lang_list {
                                let i = i.to_owned();
                                ui.selectable_value(&mut state.target_lang, i, description(i));
                            }
                        });
                    if ui.add(egui::Button::new("翻訳")).clicked() {
                        _ = task_chan.send(());
                    };
                });

                ui.separator();

                egui::ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        ui.add(
                            egui::TextEdit::multiline(&mut state.text)
                                .desired_width(2000.0)
                                .desired_rows(7)
                                .frame(false)
                                .lock_focus(true),
                        )
                    });
            });
        });

        if state.source_lang != old_source_lang || state.target_lang != old_target_lang {
            _ = task_chan.send(());
        };

        ctx.request_repaint();

        #[cfg(windows)]
        frame.set_window_size(ctx.used_size());
    }
}

pub fn get_icon_data() -> eframe::IconData {
    let ioc_buf = Cursor::new(include_bytes!("../res/translator.ico"));
    let icon_dir = ico::IconDir::read(ioc_buf).unwrap();
    let image = icon_dir.entries()[0].decode().unwrap();
    eframe::IconData {
        rgba: std::vec::Vec::from(image.rgba_data()),
        width: image.width(),
        height: image.height(),
    }
}
