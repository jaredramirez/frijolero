use bevy::time::Timer;

pub trait TimerHelper {
    fn restart(&mut self);
    fn is_stopped(&mut self) -> bool;
}
impl TimerHelper for Timer {
    fn restart(&mut self) {
        self.reset();
        self.unpause();
    }
    fn is_stopped(&mut self) -> bool {
        return self.paused() || self.finished();
    }
}
