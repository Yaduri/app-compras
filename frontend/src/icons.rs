//! Ícones de traço único (desenhos no estilo Lucide), no lugar de emojis e glifos de texto.

use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum IconKind {
    X,
    Plus,
    Copy,
    ChevronLeft,
    Cart,
    Trash,
    Check,
}

#[component]
pub fn Icon(kind: IconKind, #[props(default = 20)] size: u32, #[props(default = 2.0)] stroke: f32) -> Element {
    let body = match kind {
        IconKind::X => rsx! {
            path { d: "M18 6 6 18" }
            path { d: "m6 6 12 12" }
        },
        IconKind::Plus => rsx! {
            path { d: "M5 12h14" }
            path { d: "M12 5v14" }
        },
        IconKind::Copy => rsx! {
            rect { x: "9", y: "9", width: "13", height: "13", rx: "2" }
            path { d: "M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" }
        },
        IconKind::ChevronLeft => rsx! {
            path { d: "m15 18-6-6 6-6" }
        },
        IconKind::Cart => rsx! {
            circle { cx: "8", cy: "21", r: "1" }
            circle { cx: "19", cy: "21", r: "1" }
            path { d: "M2.05 2.05h2l2.66 12.42a2 2 0 0 0 2 1.58h9.78a2 2 0 0 0 1.95-1.57l1.65-7.43H5.12" }
        },
        IconKind::Trash => rsx! {
            path { d: "M3 6h18" }
            path { d: "M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6" }
            path { d: "M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" }
        },
        IconKind::Check => rsx! {
            path { d: "M20 6 9 17l-5-5" }
        },
    };

    rsx! {
        svg {
            class: "icon",
            width: "{size}",
            height: "{size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "{stroke}",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            "focusable": "false",
            {body}
        }
    }
}
