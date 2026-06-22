use crate::{
    collab::use_collab_websocket,
    preview::Preview,
    services::ApiService,
    toolbar::{apply_format, Toolbar},
};
use gloo_timers::callback::Timeout;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct EditorProps {
    pub notepad_id: String,
    pub preview_mode: String,
    pub save_interval: u64,
    pub disable_print_expand: bool,
}

fn save_notepad(id: String, content: String, status: UseStateHandle<String>) {
    status.set("saving".to_string());
    spawn_local(async move {
        if ApiService::save_notes(&id, &content).await.is_ok() {
            status.set("saved".to_string());
        }
    });
}

#[function_component(Editor)]
pub fn editor(props: &EditorProps) -> Html {
    let content = use_state(|| "".to_string());
    let last_loaded_id = use_state(|| "".to_string());
    let debounce_timer = use_mut_ref(|| None::<Timeout>);
    let editor_ref = use_node_ref();
    let save_status = use_state(|| "saved".to_string());
    let copy_status = use_state(|| "idle".to_string());
    let locale = use_context::<crate::i18n::LocaleContext>().unwrap();

    {
        let content = content.clone();
        let last_id = last_loaded_id.clone();
        let current_id = props.notepad_id.clone();

        use_effect_with(current_id.clone(), move |nid| {
            let nid = nid.clone();
            spawn_local(async move {
                if let Ok(res) = ApiService::get_notes(&nid).await {
                    content.set(res.content);
                    last_id.set(nid);
                }
            });
            || ()
        });
    }

    let (on_local_change, on_cursor_move) =
        use_collab_websocket(&props.notepad_id, content.clone(), editor_ref.clone());

    let on_keydown = {
        let (nid, timer, status, content) = (
            props.notepad_id.clone(),
            debounce_timer.clone(),
            save_status.clone(),
            content.clone(),
        );
        Callback::from(move |e: KeyboardEvent| {
            if (e.ctrl_key() || e.meta_key()) && e.key().to_lowercase() == "s" {
                e.prevent_default();
                if let Some(t) = timer.borrow_mut().take() {
                    t.cancel();
                }
                save_notepad(nid.clone(), (*content).clone(), status.clone());
            }
        })
    };

    let on_input = {
        let content = content.clone();
        let notepad_id = props.notepad_id.clone();
        let save_interval = props.save_interval;
        let timer_ref = debounce_timer.clone();
        let save_status = save_status.clone();
        let on_local_change = on_local_change.clone();
        let on_cursor_move = on_cursor_move.clone();

        Callback::from(move |e: InputEvent| {
            let textarea: web_sys::HtmlTextAreaElement = e.target_unchecked_into();
            let val = textarea.value();
            let old_val = (*content).clone();
            on_local_change.emit((old_val, val.clone()));
            if let Some(pos) = textarea.selection_start().ok().flatten() {
                on_cursor_move.emit(pos as usize);
            }
            content.set(val.clone());
            save_status.set("unsaved".to_string());
            if let Some(t) = timer_ref.borrow_mut().take() {
                t.cancel();
            }
            if save_interval > 0 {
                let nid = notepad_id.clone();
                let save_val = val.clone();
                let status = save_status.clone();
                let new_timer = Timeout::new(save_interval as u32, move || {
                    save_notepad(nid, save_val, status);
                });
                *timer_ref.borrow_mut() = Some(new_timer);
            }
        })
    };

    let update_cursor_pos = {
        let r = editor_ref.clone();
        let m = on_cursor_move.clone();
        move || {
            let _ = r.cast::<web_sys::HtmlTextAreaElement>().map(|t| {
                t.selection_start()
                    .ok()
                    .flatten()
                    .map(|p| m.emit(p as usize))
            });
        }
    };
    let on_click = {
        let u = update_cursor_pos.clone();
        Callback::from(move |_: MouseEvent| u())
    };
    let on_keyup = {
        let u = update_cursor_pos.clone();
        Callback::from(move |_: KeyboardEvent| u())
    };
    let on_scroll = {
        let u = update_cursor_pos;
        Callback::from(move |_: Event| u())
    };

    let on_blur = {
        let notepad_id = props.notepad_id.clone();
        let content = content.clone();
        let timer_ref = debounce_timer.clone();
        let save_status = save_status.clone();
        Callback::from(move |_| {
            if let Some(t) = timer_ref.borrow_mut().take() {
                t.cancel();
                save_notepad(notepad_id.clone(), (*content).clone(), save_status.clone());
            }
        })
    };

    let on_toolbar_click = {
        let editor_ref = editor_ref.clone();
        let content = content.clone();
        let on_local_change = on_local_change.clone();
        let save_status = save_status.clone();
        let timer_ref = debounce_timer.clone();
        let notepad_id = props.notepad_id.clone();
        let save_interval = props.save_interval;
        Callback::from(move |format_type: String| {
            if let Some(textarea) = editor_ref.cast::<web_sys::HtmlTextAreaElement>() {
                let start = textarea.selection_start().ok().flatten().unwrap_or(0) as usize;
                let end = textarea.selection_end().ok().flatten().unwrap_or(0) as usize;
                let old_val = textarea.value();
                let (new_val, new_start, new_end) =
                    apply_format(&old_val, start, end, &format_type);
                textarea.set_value(&new_val);
                on_local_change.emit((old_val, new_val.clone()));
                content.set(new_val.clone());
                let _ = textarea.focus();
                let _ = textarea.set_selection_range(new_start as u32, new_end as u32);
                save_status.set("unsaved".to_string());
                if let Some(t) = timer_ref.borrow_mut().take() {
                    t.cancel();
                }
                if save_interval > 0 {
                    let (nid, s_val, status) = (notepad_id.clone(), new_val, save_status.clone());
                    *timer_ref.borrow_mut() = Some(Timeout::new(save_interval as u32, move || {
                        save_notepad(nid, s_val, status);
                    }));
                }
            }
        })
    };

    let on_copy = {
        let (c, status) = (content.clone(), copy_status.clone());
        Callback::from(move |_| {
            if let Some(w) = web_sys::window() {
                let _ = w.navigator().clipboard().write_text(&c);
                status.set("copied".to_string());
                let s = status.clone();
                let _ = Timeout::new(2000, move || s.set("idle".to_string())).forget();
            }
        })
    };

    let on_export = {
        let (c, nid) = (content.clone(), props.notepad_id.clone());
        Callback::from(move |_| {
            if let Some(d) = web_sys::window().and_then(|w| w.document()) {
                let encoded =
                    percent_encoding::utf8_percent_encode(&c, percent_encoding::NON_ALPHANUMERIC)
                        .to_string();
                if let Ok(a) = d
                    .create_element("a")
                    .map(|a| a.unchecked_into::<web_sys::HtmlElement>())
                {
                    let _ = a.set_attribute(
                        "href",
                        &format!("data:text/markdown;charset=utf-8,{}", encoded),
                    );
                    let _ = a.set_attribute("download", &format!("{}.md", nid));
                    a.click();
                }
            }
        })
    };

    let (copy_icon, copy_text_style, copy_text) = if *copy_status == "copied" {
        (
            html! { <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" style="margin-right: 4px; color: #10b981;"><polyline points="20 6 9 17 4 12"></polyline></svg> },
            Some("color: #10b981;"),
            locale.t("copied"),
        )
    } else {
        (
            html! { <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" style="margin-right: 4px;"><rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path></svg> },
            None,
            locale.t("copy"),
        )
    };

    let show_editor = props.preview_mode != "preview-only";
    let show_preview = props.preview_mode != "off";

    let wrapper_class = match props.preview_mode.as_str() {
        "split" => "editor-preview-wrapper split-view",
        "preview-only" => "editor-preview-wrapper preview-only",
        _ => "editor-preview-wrapper",
    };

    html! {
        <div id="editor-preview-wrapper" class={wrapper_class}>
            if show_editor {
                <div id="editor-container" class="editor-container">
                    <Toolbar on_click={on_toolbar_click} />
                    <textarea
                        id="editor"
                        ref={editor_ref}
                        placeholder={locale.t("placeholder")}
                        spellcheck="true"
                        value={(*content).clone()}
                        oninput={on_input}
                        onblur={on_blur}
                        onkeydown={on_keydown}
                        onclick={on_click}
                        onkeyup={on_keyup}
                        onscroll={on_scroll}
                        autofocus=true
                    />
                    <div class="editor-actions">
                        <button class="action-button copy-button" onclick={on_copy} data-tooltip={locale.t("copy")}>
                            {copy_icon}
                            <span style={copy_text_style}>{copy_text}</span>
                        </button>
                        <button class="action-button export-button" onclick={on_export} data-tooltip={locale.t("export")}>
                            <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" style="margin-right: 4px;"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path><polyline points="7 10 12 15 17 10"></polyline><line x1="12" y1="15" x2="12" y2="3"></line></svg>
                            <span>{locale.t("export")}</span>
                        </button>
                    </div>
                    <div class={classes!("save-status", (*save_status).clone())}>
                        {
                            match save_status.as_str() {
                                "unsaved" => html! { <>{"● "}{locale.t("unsaved_changes")}</> },
                                "saving" => html! { <>{"◌ "}{locale.t("saving")}</> },
                                _ => html! { <>{"✓ "}{locale.t("saved")}</> },
                            }
                        }
                    </div>
                </div>
            }
            if show_preview {
                <Preview content={(*content).clone()} is_visible={show_preview} />
            }
        </div>
    }
}
