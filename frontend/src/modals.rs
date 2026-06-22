use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct RenameModalProps {
    pub is_open: bool,
    pub initial_value: String,
    pub on_close: Callback<()>,
    pub on_confirm: Callback<String>,
}

#[function_component(RenameModal)]
pub fn rename_modal(props: &RenameModalProps) -> Html {
    let rename_value = use_state(|| props.initial_value.clone());
    let locale = use_context::<crate::i18n::LocaleContext>().unwrap();

    {
        let rename_value = rename_value.clone();
        let initial_value = props.initial_value.clone();
        use_effect_with(initial_value, move |val| {
            rename_value.set(val.clone());
            || ()
        });
    }

    if !props.is_open {
        return html! {};
    }

    let on_input = {
        let rename_value = rename_value.clone();
        Callback::from(move |e: InputEvent| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            rename_value.set(input.value());
        })
    };

    let on_confirm_click = {
        let on_confirm = props.on_confirm.clone();
        let rename_value = rename_value.clone();
        Callback::from(move |_| {
            on_confirm.emit((*rename_value).clone());
        })
    };

    let on_cancel_click = {
        let on_close = props.on_close.clone();
        Callback::from(move |_| {
            on_close.emit(());
        })
    };

    html! {
        <div id="rename-modal" class="modal visible">
            <div class="modal-content">
                <h2>{locale.t("rename_title")}</h2>
                <input
                    type="text"
                    class="modal-input"
                    value={(*rename_value).clone()}
                    oninput={on_input}
                />
                <div class="modal-buttons">
                    <button onclick={on_cancel_click}>{locale.t("cancel")}</button>
                    <button onclick={on_confirm_click}>{locale.t("rename_confirm")}</button>
                </div>
            </div>
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct DeleteModalProps {
    pub is_open: bool,
    pub on_close: Callback<()>,
    pub on_confirm: Callback<()>,
}

#[function_component(DeleteModal)]
pub fn delete_modal(props: &DeleteModalProps) -> Html {
    let locale = use_context::<crate::i18n::LocaleContext>().unwrap();

    if !props.is_open {
        return html! {};
    }

    let on_cancel_click = {
        let on_close = props.on_close.clone();
        Callback::from(move |_| {
            on_close.emit(());
        })
    };

    let on_confirm_click = {
        let on_confirm = props.on_confirm.clone();
        Callback::from(move |_| {
            on_confirm.emit(());
        })
    };

    html! {
        <div id="delete-modal" class="modal visible">
            <div class="modal-content">
                <h2>{locale.t("delete_title")}</h2>
                <p class="modal-message">{locale.t("delete_msg")}</p>
                <div class="modal-buttons">
                    <button onclick={on_cancel_click}>{locale.t("cancel")}</button>
                    <button class="danger" onclick={on_confirm_click}>{locale.t("delete_confirm")}</button>
                </div>
            </div>
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct ShortcutsModalProps {
    pub is_open: bool,
    pub on_close: Callback<()>,
}

#[function_component(ShortcutsModal)]
pub fn shortcuts_modal(props: &ShortcutsModalProps) -> Html {
    let locale = use_context::<crate::i18n::LocaleContext>().unwrap();

    if !props.is_open {
        return html! {};
    }

    let on_close_click = {
        let on_close = props.on_close.clone();
        Callback::from(move |_| {
            on_close.emit(());
        })
    };

    html! {
        <div id="shortcuts-modal" class="modal visible" onclick={on_close_click.clone()}>
            <div class="modal-content" onclick={|e: MouseEvent| e.stop_propagation()}>
                <h2>{locale.t("shortcuts_title")}</h2>
                <ul style="list-style-type: none; padding: 0; margin: 15px 0; font-family: monospace; display: flex; flex-direction: column; gap: 10px;">
                    <li style="display: flex; justify-content: space-between; align-items: center; border-bottom: 0.1px solid var(--secondary-color); padding-bottom: 5px;">
                        <span>{locale.t("sc_search")}</span>
                        <kbd style="background: var(--secondary-color); padding: 2px 6px; border-radius: 4px; box-shadow: 0 1px 1px rgba(0,0,0,0.2);">{"Ctrl + F"}</kbd>
                    </li>
                    <li style="display: flex; justify-content: space-between; align-items: center; border-bottom: 0.1px solid var(--secondary-color); padding-bottom: 5px;">
                        <span>{locale.t("sc_save")}</span>
                        <kbd style="background: var(--secondary-color); padding: 2px 6px; border-radius: 4px; box-shadow: 0 1px 1px rgba(0,0,0,0.2);">{"Ctrl + S"}</kbd>
                    </li>
                    <li style="display: flex; justify-content: space-between; align-items: center; border-bottom: 0.1px solid var(--secondary-color); padding-bottom: 5px;">
                        <span>{locale.t("sc_preview")}</span>
                        <kbd style="background: var(--secondary-color); padding: 2px 6px; border-radius: 4px; box-shadow: 0 1px 1px rgba(0,0,0,0.2);">{"Ctrl + Shift + P"}</kbd>
                    </li>
                    <li style="display: flex; justify-content: space-between; align-items: center; border-bottom: 0.1px solid var(--secondary-color); padding-bottom: 5px;">
                        <span>{locale.t("sc_new")}</span>
                        <kbd style="background: var(--secondary-color); padding: 2px 6px; border-radius: 4px; box-shadow: 0 1px 1px rgba(0,0,0,0.2);">{"Ctrl + Alt + N"}</kbd>
                    </li>
                    <li style="display: flex; justify-content: space-between; align-items: center; border-bottom: 0.1px solid var(--secondary-color); padding-bottom: 5px;">
                        <span>{locale.t("sc_help")}</span>
                        <kbd style="background: var(--secondary-color); padding: 2px 6px; border-radius: 4px; box-shadow: 0 1px 1px rgba(0,0,0,0.2);">{"?"}</kbd>
                    </li>
                </ul>
                <div class="modal-buttons" style="justify-content: center; margin-top: 20px;">
                    <button onclick={on_close_click}>{locale.t("close")}</button>
                </div>
            </div>
        </div>
    }
}
