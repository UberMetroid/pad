use wasm_bindgen::JsCast;
use yew::prelude::*;

pub fn register_keyboard_shortcuts(
    authenticated: UseStateHandle<bool>,
    search_open: UseStateHandle<bool>,
    shortcuts_open: UseStateHandle<bool>,
    preview_mode: UseStateHandle<String>,
    on_new_notepad: Callback<MouseEvent>,
) {
    use_effect_with((*authenticated).clone(), move |&auth| {
        let on_keydown = if auth {
            let search_open = search_open.clone();
            let shortcuts_open = shortcuts_open.clone();
            let preview_mode = preview_mode.clone();
            let on_new_notepad = on_new_notepad.clone();

            let cb = wasm_bindgen::prelude::Closure::<dyn FnMut(web_sys::KeyboardEvent)>::new(
                move |e: web_sys::KeyboardEvent| {
                    let ctrl = e.ctrl_key() || e.meta_key();
                    let alt = e.alt_key();
                    let shift = e.shift_key();
                    let key = e.key();

                    if key == "?" {
                        if let Some(target) = e.target() {
                            if let Some(el) = target.dyn_ref::<web_sys::HtmlElement>() {
                                let tag = el.tag_name().to_lowercase();
                                if tag == "input" || tag == "textarea" {
                                    return;
                                }
                            }
                        }
                        e.prevent_default();
                        shortcuts_open.set(!*shortcuts_open);
                    } else if ctrl && key == "f" {
                        e.prevent_default();
                        search_open.set(true);
                    } else if ctrl && shift && key.to_lowercase() == "p" {
                        e.prevent_default();
                        let next = match preview_mode.as_str() {
                            "off" => "split",
                            "split" => "preview-only",
                            _ => "off",
                        };
                        preview_mode.set(next.to_string());
                    } else if ctrl && alt && key.to_lowercase() == "n" {
                        e.prevent_default();
                        on_new_notepad.emit(web_sys::MouseEvent::new("click").unwrap().into());
                    }
                },
            );
            let target = web_sys::window().unwrap();
            let _ = target.add_event_listener_with_callback("keydown", cb.as_ref().unchecked_ref());
            Some(cb)
        } else {
            None
        };

        move || {
            if let Some(ref cb) = on_keydown {
                if let Some(target) = web_sys::window() {
                    let _ = target.remove_event_listener_with_callback(
                        "keydown",
                        cb.as_ref().unchecked_ref(),
                    );
                }
            }
        }
    });
}
