mod types;
mod services;
mod login;
mod settings;
mod preview;
mod search;
mod editor;

use yew::prelude::*;
use wasm_bindgen_futures::spawn_local;
use web_sys::window;

use types::Notepad;
use services::{ApiService, StorageService};
use login::Login;
use settings::SettingsModal;
use search::SearchModal;
use editor::Editor;

#[function_component(App)]
pub fn app() -> Html {
    let authenticated = use_state(|| false);
    let notepads = use_state(|| Vec::<Notepad>::new());
    let active_notepad_id = use_state(|| "default".to_string());
    let settings = use_state(StorageService::get_settings);
    let preview_mode = use_state(|| "off".to_string());
    let search_open = use_state(|| false);
    let settings_open = use_state(|| false);
    let rename_open = use_state(|| false);
    let delete_open = use_state(|| false);
    let rename_value = use_state(|| "".to_string());
    let app_version = use_state(|| "1.0.5".to_string());

    {
        let authenticated = authenticated.clone();
        let notepads = notepads.clone();
        let active_id = active_notepad_id.clone();
        let preview = preview_mode.clone();
        let s = settings.clone();
        let version = app_version.clone();
        
        use_effect_with((*authenticated).clone(), move |&auth| {
            if auth {
                spawn_local(async move {
                    if let Ok(config) = ApiService::get_config().await {
                        version.set(config.version);
                    }
                    if let Ok(res) = ApiService::get_notepads().await {
                        notepads.set(res.notepads_list);
                        active_id.set(res.note_history);
                    }
                    let cur_s = StorageService::get_settings();
                    preview.set(cur_s.default_markdown_preview_mode.clone());
                    s.set(cur_s);
                });
            }
            || ()
        });
    }

    if !*authenticated {
        return html! {
            <Login on_login_success={Callback::from(move |_| authenticated.set(true))} />
        };
    }

    let on_notepad_select = {
        let active_id = active_notepad_id.clone();
        Callback::from(move |e: Event| {
            let select: web_sys::HtmlSelectElement = e.target_unchecked_into();
            active_id.set(select.value());
        })
    };

    let on_new_notepad = {
        let notepads = notepads.clone();
        let active_id = active_notepad_id.clone();
        Callback::from(move |_| {
            let notepads = notepads.clone();
            let active_id = active_id.clone();
            spawn_local(async move {
                if let Ok(new_note) = ApiService::create_notepad().await {
                    active_id.set(new_note.id.clone());
                    if let Ok(res) = ApiService::get_notepads().await {
                        notepads.set(res.notepads_list);
                    }
                }
            });
        })
    };

    let on_rename_confirm = {
        let notepad_id = (*active_notepad_id).clone();
        let rename_val = rename_value.clone();
        let rename_open = rename_open.clone();
        let notepads = notepads.clone();
        Callback::from(move |_| {
            let nid = notepad_id.clone();
            let val = (*rename_val).clone();
            let rename_open = rename_open.clone();
            let notepads = notepads.clone();
            spawn_local(async move {
                let _ = ApiService::rename_notepad(&nid, &val).await;
                rename_open.set(false);
                if let Ok(res) = ApiService::get_notepads().await {
                    notepads.set(res.notepads_list);
                }
            });
        })
    };

    let on_delete_confirm = {
        let notepad_id = (*active_notepad_id).clone();
        let delete_open = delete_open.clone();
        let active_id = active_notepad_id.clone();
        let notepads = notepads.clone();
        Callback::from(move |_| {
            let nid = notepad_id.clone();
            let delete_open = delete_open.clone();
            let active_id = active_id.clone();
            let notepads = notepads.clone();
            spawn_local(async move {
                let _ = ApiService::delete_notepad(&nid).await;
                delete_open.set(false);
                active_id.set("default".to_string());
                if let Ok(res) = ApiService::get_notepads().await {
                    notepads.set(res.notepads_list);
                }
            });
        })
    };

    let toggle_theme = Callback::from(move |_| {
        let current = StorageService::get_theme();
        let next = if current == "dark" { "light" } else { "dark" };
        StorageService::set_theme(next);
        if let Some(doc) = window().and_then(|w| w.document()) {
            if let Some(root) = doc.document_element() {
                let _ = root.set_attribute("data-theme", next);
            }
        }
    });

    let current_theme = StorageService::get_theme();
    let theme_toggle_icon = if current_theme == "dark" {
        html! { <svg id="sun-icon" class="sun" width="24" height="24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14.828 14.828a4 4 0 1 0 -5.656 -5.656a4 4 0 0 0 5.656 5.656z" /><path d="M6.343 17.657l-1.414 1.414" /><path d="M6.343 6.343l-1.414 -1.414" /><path d="M17.657 6.343l1.414 -1.414" /><path d="M17.657 17.657l1.414 1.414" /><path d="M4 12h-2" /><path d="M12 4v-2" /><path d="M20 12h2" /><path d="M12 20v2" /></svg> }
    } else {
        html! { <svg id="moon-icon" class="moon" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 3c.132 0 .263 0 .393 0a7.5 7.5 0 0 0 7.92 12.446a9 9 0 1 1 -8.313 -12.454z" /></svg> }
    };

    let active_name = notepads.iter().find(|n| n.id == *active_notepad_id).map(|n| n.name.clone()).unwrap_or_else(|| "Default".to_string());

    html! {
        <div class="container">
            <header>
                <div class="header-top">
                    <div id="header-title" data-tooltip={format!("Version: {}", *app_version)}>
                        <h1 style="font-size: 1.5rem;">{"RustPad"}</h1>
                    </div>
                    <div class="header-right">
                        <button id="search-open" class="icon-button" onclick={let s = search_open.clone(); move |_| s.set(true)} data-tooltip="Search">
                            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M10 10m-7 0a7 7 0 1 0 14 0a7 7 0 1 0 -14 0" /><path d="M21 21l-6 -6" /></svg>
                        </button>
                        <button id="settings-button" class="icon-button" onclick={let s = settings_open.clone(); move |_| s.set(true)} data-tooltip="Settings">
                            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M14 6m-2 0a2 2 0 1 0 4 0a2 2 0 1 0 -4 0" /><path d="M4 6l8 0" /><path d="M16 6l4 0" /><path d="M8 12m-2 0a2 2 0 1 0 4 0a2 2 0 1 0 -4 0" /><path d="M4 12l2 0" /><path d="M10 12l10 0" /><path d="M17 18m-2 0a2 2 0 1 0 4 0a2 2 0 1 0 -4 0" /><path d="M4 18l11 0" /><path d="M19 18l1 0" /></svg>
                        </button>
                        <button id="theme-toggle" class="icon-button" onclick={toggle_theme}>
                            {theme_toggle_icon}
                        </button>
                    </div>
                </div>
                <div class="notepad-controls">
                    <div class="select-wrapper">
                        <button id="new-notepad" class="icon-button" onclick={on_new_notepad} aria-label="Create new notepad">
                            <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="12" y1="5" x2="12" y2="19"></line><line x1="5" y1="12" x2="19" y2="12"></line></svg>
                        </button>
                        <select id="notepad-selector" onchange={on_notepad_select} value={(*active_notepad_id).clone()}>
                            {
                                for notepads.iter().map(|n| {
                                    html! { <option value={n.id.clone()}>{&n.name}</option> }
                                })
                            }
                        </select>
                    </div>
                    <div class="notepad-controls-wrapper">
                        <button id="rename-notepad" class="icon-button" onclick={let r = rename_open.clone(); let rv = rename_value.clone(); let name = active_name.clone(); move |_| { r.set(true); rv.set(name.clone()) }}>
                            <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M17 3a2.85 2.83 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5L17 3z"></path></svg>
                        </button>
                        <button id="delete-notepad" class="icon-button" onclick={let d = delete_open.clone(); move |_| d.set(true)}>
                            <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M3 6h18"></path><path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6"></path><path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2"></path></svg>
                        </button>
                        <button id="preview-markdown" class="icon-button" onclick={let p = preview_mode.clone(); move |_| {
                            let next = match p.as_str() {
                                "off" => "split",
                                "split" => "preview-only",
                                _ => "off",
                            };
                            p.set(next.to_string());
                        }}>
                            <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.4"><path d="M3 5m0 2a2 2 0 0 1 2 -2h14a2 2 0 0 1 2 2v10a2 2 0 0 1 -2 2h-14a2 2 0 0 1 -2 -2z" /><path d="M7 15v-6l2 2l2 -2v6" /><path d="M14 13l2 2l2 -2m-2 2v-6" /></svg>
                        </button>
                    </div>
                </div>
            </header>
            <main>
                <Editor 
                    notepad_id={(*active_notepad_id).clone()}
                    preview_mode={(*preview_mode).clone()}
                    save_interval={settings.save_status_message_interval}
                    disable_print_expand={settings.disable_print_expand}
                />
            </main>
            
            <SearchModal 
                is_open={*search_open}
                on_close={let s = search_open.clone(); Callback::from(move |_| s.set(false))}
                on_select={let active_id = active_notepad_id.clone(); Callback::from(move |id| active_id.set(id))}
            />
            
            <SettingsModal 
                is_open={*settings_open}
                on_close={let s = settings_open.clone(); Callback::from(move |_| s.set(false))}
                on_save={let s = settings.clone(); Callback::from(move |new_s| s.set(new_s))}
            />

            if *rename_open {
                <div id="rename-modal" class="modal" style="display: block;">
                    <div class="modal-content">
                        <h2>{"Rename Notepad"}</h2>
                        <input 
                            type="text" 
                            class="modal-input" 
                            value={(*rename_value).clone()}
                            oninput={let r = rename_value.clone(); Callback::from(move |e: InputEvent| {
                                let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                r.set(input.value());
                            })}
                        />
                        <div class="modal-buttons">
                            <button onclick={let r = rename_open.clone(); move |_| r.set(false)}>{"Cancel"}</button>
                            <button onclick={on_rename_confirm}>{"Rename"}</button>
                        </div>
                    </div>
                </div>
            }

            if *delete_open {
                <div id="delete-modal" class="modal" style="display: block;">
                    <div class="modal-content">
                        <h2>{"Delete Notepad"}</h2>
                        <p class="modal-message">{"Are you sure you want to delete this notepad? This action cannot be undone."}</p>
                        <div class="modal-buttons">
                            <button onclick={let d = delete_open.clone(); move |_| d.set(false)}>{"Cancel"}</button>
                            <button class="danger" onclick={on_delete_confirm}>{"Delete"}</button>
                        </div>
                    </div>
                </div>
            }
        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
