use yew::prelude::*;
use wasm_bindgen_futures::spawn_local;
use gloo_timers::callback::Timeout;
use gloo_net::websocket::futures::WebSocket;
use futures_util::{SinkExt, StreamExt};
use serde_json::json;

use crate::services::ApiService;
use crate::preview::Preview;

#[derive(Properties, PartialEq)]
pub struct EditorProps {
    pub notepad_id: String,
    pub preview_mode: String,
    pub save_interval: u64,
    pub disable_print_expand: bool,
}

#[function_component(Editor)]
pub fn editor(props: &EditorProps) -> Html {
    let content = use_state(|| "".to_string());
    let last_loaded_id = use_state(|| "".to_string());
    let debounce_timer = use_mut_ref(|| None::<Timeout>);
    let editor_ref = use_node_ref();

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

    {
        let current_id = props.notepad_id.clone();
        
        use_effect_with(current_id.clone(), move |nid| {
            let nid = nid.clone();
            let window = web_sys::window().unwrap();
            let protocol = if window.location().protocol().unwrap() == "https:" { "wss:" } else { "ws:" };
            let host = window.location().host().unwrap();
            let ws_url = format!("{}//{}/ws", protocol, host);
            
            if let Ok(ws) = WebSocket::open(&ws_url) {
                let (mut write, mut read) = ws.split();
                let user_id = format!("user_{}", chrono::Utc::now().timestamp_millis());
                let init_msg = json!({
                    "type": "sync_request",
                    "userId": user_id,
                    "notepadId": nid
                }).to_string();
                
                spawn_local(async move {
                    let _ = write.send(gloo_net::websocket::Message::Text(init_msg)).await;
                });
                
                spawn_local(async move {
                    while let Some(Ok(msg)) = read.next().await {
                        if let gloo_net::websocket::Message::Text(_) = msg {}
                    }
                });
            }
            
            || ()
        });
    }

    let on_input = {
        let content = content.clone();
        let notepad_id = props.notepad_id.clone();
        let save_interval = props.save_interval;
        let timer_ref = debounce_timer.clone();
        
        Callback::from(move |e: InputEvent| {
            let textarea: web_sys::HtmlTextAreaElement = e.target_unchecked_into();
            let val = textarea.value();
            content.set(val.clone());
            
            if let Some(t) = timer_ref.borrow_mut().take() {
                t.cancel();
            }
            
            if save_interval > 0 {
                let nid = notepad_id.clone();
                let save_val = val.clone();
                let new_timer = Timeout::new(save_interval as u32, move || {
                    spawn_local(async move {
                        let _ = ApiService::save_notes(&nid, &save_val).await;
                    });
                });
                *timer_ref.borrow_mut() = Some(new_timer);
            }
        })
    };

    let on_blur = {
        let notepad_id = props.notepad_id.clone();
        let content_val = (*content).clone();
        let timer_ref = debounce_timer.clone();
        
        Callback::from(move |_| {
            if let Some(t) = timer_ref.borrow_mut().take() {
                t.cancel();
                let nid = notepad_id.clone();
                let save_val = content_val.clone();
                spawn_local(async move {
                    let _ = ApiService::save_notes(&nid, &save_val).await;
                });
            }
        })
    };

    let show_editor = props.preview_mode != "preview-only";
    let show_preview = props.preview_mode != "off";

    html! {
        <div id="editor-preview-wrapper" class="editor-preview-wrapper">
            if show_editor {
                <div id="editor-container" class={classes!("editor-container", if props.preview_mode == "split" { Some("split-view") } else { None })}>
                    <textarea 
                        id="editor" 
                        ref={editor_ref}
                        placeholder="Start typing your notes here..." 
                        spellcheck="true" 
                        value={(*content).clone()}
                        oninput={on_input}
                        onblur={on_blur}
                        autofocus=true
                    />
                </div>
            }
            
            if show_preview {
                <Preview 
                    content={(*content).clone()} 
                    is_visible={show_preview} 
                />
            }
        </div>
    }
}
