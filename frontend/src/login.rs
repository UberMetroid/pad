use yew::prelude::*;
use wasm_bindgen_futures::spawn_local;
use crate::services::{ApiService, StorageService};

#[derive(Properties, PartialEq)]
pub struct LoginProps {
    pub on_login_success: Callback<()>,
}

#[function_component(Login)]
pub fn login(props: &LoginProps) -> Html {
    let pin_input = use_state(|| "".to_string());
    let error_msg = use_state(|| "".to_string());
    let is_locked = use_state(|| false);
    let pin_length = use_state(|| 4);
    let theme = use_state(StorageService::get_theme);

    {
        let on_success = props.on_login_success.clone();
        let is_locked = is_locked.clone();
        let pin_length = pin_length.clone();
        use_effect_with((), move |_| {
            spawn_local(async move {
                if let Ok(res) = ApiService::check_pin_required().await {
                    if !res.required {
                        on_success.emit(());
                    } else {
                        is_locked.set(res.locked);
                        pin_length.set(res.length);
                    }
                }
            });
            || ()
        });
    }

    let toggle_theme = {
        let theme = theme.clone();
        Callback::from(move |_| {
            let next = if *theme == "dark" { "light" } else { "dark" };
            StorageService::set_theme(next);
            if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
                if let Some(root) = doc.document_element() {
                    let _ = root.set_attribute("data-theme", next);
                }
            }
            theme.set(next.to_string());
        })
    };

    let on_input = {
        let pin_input = pin_input.clone();
        let pin_len = *pin_length;
        let on_success = props.on_login_success.clone();
        let error_msg = error_msg.clone();
        let is_locked = is_locked.clone();
        
        Callback::from(move |e: InputEvent| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            let val = input.value();
            if val.len() <= pin_len {
                pin_input.set(val.clone());
                error_msg.set("".to_string());
                
                if val.len() == pin_len {
                    let on_success = on_success.clone();
                    let error_msg = error_msg.clone();
                    let is_locked = is_locked.clone();
                    let val_clone = val.clone();
                    
                    spawn_local(async move {
                        if let Ok(res) = ApiService::verify_pin(&val_clone).await {
                            if res.success {
                                on_success.emit(());
                            } else {
                                if let Some(err) = res.error {
                                    if err.contains("Too many attempts") {
                                        is_locked.set(true);
                                    }
                                    error_msg.set(err);
                                } else {
                                    error_msg.set("Invalid PIN".to_string());
                                }
                            }
                        }
                    });
                }
            }
        })
    };

    let theme_toggle_icon = if *theme == "dark" {
        html! {
            <svg id="sun-icon" class="sun" width="24" height="24" fill="none" stroke="currentColor" stroke-width="2">
                <path stroke="none" d="M0 0h24v24H0z" fill="none"/>
                <path d="M14.828 14.828a4 4 0 1 0 -5.656 -5.656a4 4 0 0 0 5.656 5.656z" />
                <path d="M6.343 17.657l-1.414 1.414" />
                <path d="M6.343 6.343l-1.414 -1.414" />
                <path d="M17.657 6.343l1.414 -1.414" />
                <path d="M17.657 17.657l1.414 1.414" />
                <path d="M4 12h-2" />
                <path d="M12 4v-2" />
                <path d="M20 12h2" />
                <path d="M12 20v2" />
            </svg>
        }
    } else {
        html! {
            <svg id="moon-icon" class="moon" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <path stroke="none" d="M0 0h24v24H0z" fill="none"/>
                <path d="M12 3c.132 0 .263 0 .393 0a7.5 7.5 0 0 0 7.92 12.446a9 9 0 1 1 -8.313 -12.454z" />
            </svg>
        }
    };

    html! {
        <div class="container login-container">
            <button id="theme-toggle" class="theme-toggle" onclick={toggle_theme} aria-label="Toggle dark mode">
                {theme_toggle_icon}
            </button>
            <div id="login-content">
                <div class="pin-header">
                    <h1 id="site-title">{"RustPad"}</h1>
                    <h2 id="pin-description">
                        {if *is_locked { "Locked Out" } else { "Enter PIN" }}
                    </h2>
                </div>
                <div class="pin-wrapper">
                    <input 
                        type="password" 
                        class="modal-input pin-input-field" 
                        value={(*pin_input).clone()}
                        oninput={on_input}
                        disabled={*is_locked}
                        placeholder={"• ".repeat(*pin_length).trim().to_string()}
                        maxlength={pin_length.to_string()}
                        autofocus=true
                        style="text-align: center; font-size: 2rem; letter-spacing: 0.5em; width: 100%; max-width: 300px; margin: 0 auto; display: block;"
                    />
                </div>
                <p id="pin-error" class="error-message">
                    {(*error_msg).clone()}
                </p>
            </div>
        </div>
    }
}
