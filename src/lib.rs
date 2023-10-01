use std::rc::Rc;

use dioxus_core::ScopeState;
use dioxus_hooks::UnboundedReceiver;
use dioxus_hooks::{to_owned, use_coroutine, use_ref, Coroutine, RefCell, UseRef};
use easer::functions::{Easing, Linear};
use futures_util::StreamExt;
use instant::Duration;
use tokio::time::interval;
use uuid::Uuid;

pub enum TransitionPhase {
    From(f32),
    To(f32),
}

pub enum AnimationEasing {
    EaseIn,
    EaseOut,
    EaseInOut,
}

// TODO: Add more functions
pub enum Animation {
    Linear(AnimationEasing, Duration),
    Bounce(AnimationEasing, Duration),
}

impl Animation {
    fn time(&self) -> &Duration {
        match self {
            Self::Linear(_, time) => time,
            Self::Bounce(_, time) => time,
        }
    }
}

pub struct UseTransition {
    value: Rc<RefCell<f32>>,
    channel: Coroutine<(Animation, TransitionDirection)>,
    current_id: UseRef<Option<Uuid>>,
}

enum TransitionDirection {
    Forward,
    Backwards,
}

impl UseTransition {
    pub fn read(&self) -> f32 {
        *self.value.borrow()
    }

    pub fn forward(&self, animation: Animation) {
        *self.current_id.write_silent() = Some(Uuid::new_v4());
        self.channel.send((animation, TransitionDirection::Forward))
    }

    pub fn backwards(&self, animation: Animation) {
        *self.current_id.write_silent() = Some(Uuid::new_v4());
        self.channel
            .send((animation, TransitionDirection::Backwards))
    }
}

pub fn use_transition<const N: usize>(
    cx: &ScopeState,
    phases: impl FnOnce() -> [TransitionPhase; N],
) -> &UseTransition {
    let phases = cx.use_hook(phases);
    let current_id = use_ref(cx, || None);

    let (value, start_value, end_value) = cx.use_hook(|| {
        // Find the start value
        let start_value = phases
            .iter()
            .find_map(|phase| {
                if let TransitionPhase::From(val) = phase {
                    Some(*val)
                } else {
                    None
                }
            })
            .unwrap_or_default();
        // Find the end value
        let end_value = phases
            .iter()
            .find_map(|phase| {
                if let TransitionPhase::To(val) = phase {
                    Some(*val)
                } else {
                    None
                }
            })
            .unwrap_or_default();
        (Rc::new(RefCell::new(start_value)), start_value, end_value)
    });

    let channel = use_coroutine(
        cx,
        |mut rx: UnboundedReceiver<(Animation, TransitionDirection)>| {
            let schedule_update = cx.schedule_update();
            to_owned![value, current_id, start_value, end_value];
            async move {
                let mut running_id = None;
                while let Some((animation, direction)) = rx.next().await {
                    let mut current_frame: f32 = 0.0;
                    let mut ticker = interval(Duration::from_millis(1));
                    loop {
                        // Stop running this animation if there is a new one scheduled
                        if running_id.is_some() && running_id != *current_id.read() {
                            running_id = None;
                            break;
                        }
                        // Save this animation ID
                        running_id = *current_id.read();

                        // Tick 1ms
                        ticker.tick().await;
                        current_frame += 1.0;

                        // Ease the animation value
                        match &animation {
                            Animation::Bounce(_, _) => {}
                            Animation::Linear(easing, time) => {
                                let time = time.as_millis() as f32;
                                let (start_value, end_value) = match direction {
                                    TransitionDirection::Forward => {
                                        (start_value, end_value - start_value)
                                    }
                                    TransitionDirection::Backwards => {
                                        (end_value, start_value - end_value)
                                    }
                                };
                                *value.borrow_mut() = match easing {
                                    AnimationEasing::EaseIn => {
                                        Linear::ease_in(current_frame, start_value, end_value, time)
                                    }
                                    AnimationEasing::EaseInOut => Linear::ease_in_out(
                                        current_frame,
                                        start_value,
                                        end_value,
                                        time,
                                    ),
                                    AnimationEasing::EaseOut => Linear::ease_in_out(
                                        current_frame,
                                        start_value,
                                        end_value,
                                        time,
                                    ),
                                };
                                (schedule_update)();
                            }
                        }

                        let time = animation.time();

                        // Stop the animation once done
                        if current_frame >= time.as_millis() as f32 {
                            if running_id == *current_id.read() {
                                // Remove the current animation if there is no new one scheduled
                                *current_id.write_silent() = None;
                                running_id = None;
                            }
                            break;
                        }
                    }
                }
            }
        },
    );

    cx.use_hook(|| UseTransition {
        value: value.clone(),
        channel: channel.clone(),
        current_id: current_id.clone(),
    })
}
