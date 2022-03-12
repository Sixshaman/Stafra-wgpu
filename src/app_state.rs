#[derive(Copy, Clone, PartialEq)]
pub enum RunState
{
    Stopped,
    Paused,
    Running
}

pub struct AppState
{
    pub run_state:       RunState,
    pub click_rule_data: [u32; 32 * 32]
}

impl AppState
{
    pub fn new() -> Self
    {
        let mut click_rule_data = [0; 32 * 32];

        let click_rule_size = 32;
        let center_cell_x = (click_rule_size - 1) / 2;
        let center_cell_y = (click_rule_size - 1) / 2;

        click_rule_data[(center_cell_x + 0) * click_rule_size + (center_cell_y + 0)] = 1;
        click_rule_data[(center_cell_x + 1) * click_rule_size + (center_cell_y + 0)] = 1;
        click_rule_data[(center_cell_x - 1) * click_rule_size + (center_cell_y + 0)] = 1;
        click_rule_data[(center_cell_x + 0) * click_rule_size + (center_cell_y + 1)] = 1;
        click_rule_data[(center_cell_x + 0) * click_rule_size + (center_cell_y - 1)] = 1;

        Self
        {
            run_state: RunState::Stopped,
            click_rule_data
        }
    }

    pub fn board_size_from_index(index: u32) -> u32
    {
        (1 << (index + 1)) - 1
    }
}