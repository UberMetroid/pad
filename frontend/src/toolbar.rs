use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ToolbarProps {
    pub on_click: Callback<String>,
}

#[function_component(Toolbar)]
pub fn toolbar(props: &ToolbarProps) -> Html {
    let locale = use_context::<crate::i18n::LocaleContext>().unwrap();
    let buttons = vec![
        (
            "bold",
            "tb_bold",
            html! { <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><path d="M6 4h8a4 4 0 0 1 4 4 4 4 0 0 1-4 4H6z"/><path d="M6 12h9a4 4 0 0 1 4 4 4 4 0 0 1-4 4H6z"/></svg> },
        ),
        (
            "italic",
            "tb_italic",
            html! { <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="19" y1="4" x2="10" y2="4"/><line x1="14" y1="20" x2="5" y2="20"/><line x1="15" y1="4" x2="9" y2="20"/></svg> },
        ),
        (
            "header",
            "tb_heading",
            html! { <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="4" y1="12" x2="20" y2="12"/><line x1="4" y1="4" x2="4" y2="20"/><line x1="20" y1="4" x2="20" y2="20"/></svg> },
        ),
        (
            "link",
            "tb_link",
            html! { <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"/><path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"/></svg> },
        ),
        (
            "code",
            "tb_code",
            html! { <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="16 18 22 12 16 6"/><polyline points="8 6 2 12 8 18"/></svg> },
        ),
        (
            "list",
            "tb_list",
            html! { <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="9" y1="6" x2="20" y2="6"/><line x1="9" y1="12" x2="20" y2="12"/><line x1="9" y1="18" x2="20" y2="18"/><line x1="5" y1="6" x2="5.01" y2="6"/><line x1="5" y1="12" x2="5.01" y2="12"/><line x1="5" y1="18" x2="5.01" y2="18"/></svg> },
        ),
    ];

    html! {
        <div class="editor-toolbar" style="display: flex; gap: 6px; padding: 6px 12px; background: var(--header-bg); border-bottom: 1px solid var(--secondary-color); border-top-left-radius: 8px; border-top-right-radius: 8px; align-items: center;">
            {
                for buttons.into_iter().map(|(id, tooltip, icon)| {
                    let on_click_cb = props.on_click.clone();
                    let format_id = id.to_string();
                    html! {
                        <button class="toolbar-button" onclick={Callback::from(move |_| on_click_cb.emit(format_id.clone()))} data-tooltip={locale.t(tooltip)} style="background: none; border: none; padding: 4px 8px; cursor: pointer; border-radius: 4px; display: inline-flex; align-items: center; justify-content: center; color: var(--text-color); transition: background 0.2s;">
                            {icon}
                        </button>
                    }
                })
            }
        </div>
    }
}

pub fn apply_format(
    text: &str,
    start_char: usize,
    end_char: usize,
    format_type: &str,
) -> (String, usize, usize) {
    let chars: Vec<char> = text.chars().collect();
    let total_len = chars.len();
    let start = std::cmp::min(start_char, total_len);
    let end = std::cmp::min(end_char, total_len);

    let before: String = chars[..start].iter().collect();
    let selected: String = chars[start..end].iter().collect();
    let after: String = chars[end..].iter().collect();

    match format_type {
        "bold" => {
            let new_text = format!("{}**{}**{}", before, selected, after);
            let new_cursor = start + 2 + selected.chars().count() + 2;
            (new_text, new_cursor, new_cursor)
        }
        "italic" => {
            let new_text = format!("{}*{}*{}", before, selected, after);
            let new_cursor = start + 1 + selected.chars().count() + 1;
            (new_text, new_cursor, new_cursor)
        }
        "header" => {
            let new_text = format!("{}# {}{}", before, selected, after);
            let new_cursor = start + 2 + selected.chars().count();
            (new_text, new_cursor, new_cursor)
        }
        "link" => {
            let new_text = format!("{}[{}](url){}", before, selected, after);
            let new_cursor = start + 1 + selected.chars().count() + 6;
            (new_text, new_cursor, new_cursor)
        }
        "code" => {
            let new_text = format!("{}```\n{}\n```{}", before, selected, after);
            let new_cursor = start + 4 + selected.chars().count() + 4;
            (new_text, new_cursor, new_cursor)
        }
        "list" => {
            let new_text = format!("{}- {}{}", before, selected, after);
            let new_cursor = start + 2 + selected.chars().count();
            (new_text, new_cursor, new_cursor)
        }
        _ => (text.to_string(), start, end),
    }
}
