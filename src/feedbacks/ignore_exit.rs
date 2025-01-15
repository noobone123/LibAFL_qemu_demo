use std::borrow::Cow;

use libafl::{executors::ExitKind, feedbacks::{Feedback, StateInitializer}, Error, HasMetadata};
use libafl_bolts::Named;

use crate::modules::ExecMeta;

pub struct IgnoreExitFeedback;

impl<EM, I, OT, S> Feedback<EM, I, OT, S> for IgnoreExitFeedback 
where 
    S: HasMetadata,
{
    fn is_interesting(
        &mut self,
        _state: &mut S,
        _manager: &mut EM,
        _input: &I,
        _observers: &OT,
        _exit_kind: &ExitKind,
    ) -> Result<bool, Error> {
        let exec_meta = _state
            .metadata_map_mut()
            .get_mut::<ExecMeta>()
            .expect("Can't get exec_meta");
        if exec_meta.ignore {
            log::info!("IgnoreExitFeedback: ignoring exit");
            exec_meta.ignore = false;
            Ok(false)
        } else {
            log::info!("IgnoreExitFeedback: No exiting found");
            Ok(true)
        }
    }
}

/// Custom feedbacks that implement the `Feedback` trait must also
/// implement the `StateInitializer` trait and the `Named` trait.
impl<S> StateInitializer<S> for IgnoreExitFeedback {}

impl Named for IgnoreExitFeedback {
    fn name(&self) -> &Cow<'static, str> {
        static NAME: Cow<'static, str> = Cow::Borrowed("IgnoreExitFeedback");
        &NAME
    }
}
