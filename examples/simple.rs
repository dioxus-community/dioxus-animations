#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use dioxus::prelude::*;
use dioxus_animations::{use_transition, Animation, AnimationEasing, TransitionPhase};
use instant::Duration;

fn main() {
    dioxus_desktop::launch(app);
}

fn app(cx: Scope) -> Element {
    let transition = use_transition(cx, || {
        [TransitionPhase::From(0.0), TransitionPhase::To(100.0)]
    });

    let forward = |_| {
        transition.forward(Animation::Linear(
            AnimationEasing::EaseInOut,
            Duration::from_millis(400),
        ))
    };

    let backwards = |_| {
        transition.backwards(Animation::Linear(
            AnimationEasing::EaseInOut,
            Duration::from_millis(400),
        ))
    };

    render!(
        button {
            onclick: forward,
            "forward!"
        }
        button {
            onclick: backwards,
            "backwards!"
        }
        div {
            width: "{transition.read()}%",
            height: "100px",
            border_radius: "8px",
            background: "linear-gradient(90deg, rgba(255,88,0,1) 19%, rgba(134,58,152,1) 51%, rgba(0,187,255,1) 100%)",
        }
    )
}
