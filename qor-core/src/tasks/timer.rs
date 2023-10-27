use crate::{drivers::timer::HardwareTimerDriver, structures::time::Microseconds};

#[allow(clippy::module_name_repetitions)]
pub struct TimerFuture<'a, E: Copy> {
    wake_time: Microseconds,
    timer: &'a dyn HardwareTimerDriver<HardwareTimerError = E>,
    error: Option<E>,
}

impl<'a, E: Copy> TimerFuture<'a, E> {
    /// Construct a new timer future from a `HardwareTimerDriver` and a duration
    pub fn new(
        timer: &'a dyn HardwareTimerDriver<HardwareTimerError = E>,
        duration: Microseconds,
    ) -> Self {
        let (error, wake_time) = match timer.time(0.into()) {
            Ok(current_time) => (None, current_time + duration),
            Err(e) => (Some(e), Microseconds(0)),
        };

        Self {
            wake_time,
            timer,
            error,
        }
    }
}

pub fn wait<E: Copy>(
    timer: &dyn HardwareTimerDriver<HardwareTimerError = E>,
    duration: Microseconds,
) -> TimerFuture<'_, E> {
    TimerFuture::new(timer, duration)
}

impl<'a, E: Copy> core::future::Future for TimerFuture<'a, E> {
    type Output = Result<(), E>;

    fn poll(
        self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        if let Some(error) = self.error {
            core::task::Poll::Ready(Err(error))
        } else {
            match self.timer.time(0.into()) {
                Ok(t) => {
                    if t.0 > self.wake_time.0 {
                        core::task::Poll::Ready(Ok(()))
                    } else {
                        cx.waker().wake_by_ref();
                        core::task::Poll::Pending
                    }
                }
                Err(e) => core::task::Poll::Ready(Err(e)),
            }
        }
    }
}
