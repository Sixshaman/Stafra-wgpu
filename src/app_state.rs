#[derive(Copy, Clone, PartialEq)]
pub enum RunState
{
    Stopped,
    Paused,
    Running
}

pub struct AppState
{
    pub run_state: RunState
}

impl AppState
{
    pub fn new() -> Self
    {
        Self
        {
            run_state: RunState::Stopped
        }
    }
}