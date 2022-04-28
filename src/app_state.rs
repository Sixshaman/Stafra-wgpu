#[derive(Copy, Clone, PartialEq, Debug)]
pub enum RunState
{
    Stopped,
    Paused,
    Running,
    Recording,
    PausedRecording
}

pub enum ClickRuleInitData
{
    Default,
    Custom([u8; 32 * 32])
}

pub struct AppState
{
    pub run_state:       RunState,
    pub last_frame:      u32,
    pub click_rule_data: [u8; 32 * 32]
}

//Returns the position in string "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-+"
fn decode_base_64_char(ch: char) -> u8
{
    match ch
    {
        'A'..='Z' =>  0 + (ch as u8 - 'A' as u8),
        'a'..='z' => 26 + (ch as u8 - 'a' as u8),
        '0'..='9' => 52 + (ch as u8 - '0' as u8),
        '+'       => 62,
        '-'       => 63,
        _         => 0
    }
}

fn encode_base_64_char(n: u8) -> char
{
    if n > 63
    {
        return '\0';
    }

    return "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-+".as_bytes()[n as usize] as char;
}

pub fn parse_click_rule_base64(base64_click_rule: &str) -> [u8; 32 * 32]
{
    let mut result = [0u8; 32 * 32];
    let click_rule_size = 32;

    //Each character encodes 6 cells
    let click_rule_diameter = ((6.0 * base64_click_rule.len() as f64).sqrt() as u32).clamp(0, click_rule_size);
    let click_rule_start = (click_rule_size - click_rule_diameter) / 2;

    let click_rule_characters = ((click_rule_diameter * click_rule_diameter) as f32 / 6.0).ceil() as usize;
    for (index, c) in base64_click_rule[0..click_rule_characters].chars().enumerate()
    {
        let decoded_bits = decode_base_64_char(c);
        for cell_bit in 0..6
        {
            let cell_enabled = (decoded_bits >> cell_bit) & 0x1;
            if cell_enabled == 0
            {
                continue;
            }

            let total_index = (index * 6 + cell_bit) as u32;

            let encoded_y = total_index / click_rule_diameter;
            let encoded_x = total_index % click_rule_diameter;

            let click_rule_index = ((click_rule_start + encoded_y) * click_rule_size + (click_rule_start + encoded_x)) as usize;
            result[click_rule_index] = 1;
        }
    }

    result
}

impl AppState
{
    pub fn new(click_rule_init_data: ClickRuleInitData, last_frame: u32) -> Self
    {
        let mut click_rule_data = [0u8; 32 * 32];

        if let ClickRuleInitData::Custom(data_array) = click_rule_init_data
        {
            click_rule_data = data_array;
        }
        else
        {
            let click_rule_size = 32;
            let center_cell_x = (click_rule_size - 1) / 2;
            let center_cell_y = (click_rule_size - 1) / 2;

            click_rule_data[(center_cell_x + 0) * click_rule_size + (center_cell_y + 0)] = 1;
            click_rule_data[(center_cell_x + 1) * click_rule_size + (center_cell_y + 0)] = 1;
            click_rule_data[(center_cell_x - 1) * click_rule_size + (center_cell_y + 0)] = 1;
            click_rule_data[(center_cell_x + 0) * click_rule_size + (center_cell_y + 1)] = 1;
            click_rule_data[(center_cell_x + 0) * click_rule_size + (center_cell_y - 1)] = 1;
        }

        Self
        {
            run_state:  RunState::Stopped,
            last_frame,
            click_rule_data
        }
    }

    pub fn board_size_from_index(index: u32) -> u32
    {
        (1 << (index + 1)) - 1
    }

    pub fn encode_click_rule_base64(&self) -> String
    {
        let click_rule_size   = 32;
        let click_rule_center = (click_rule_size - 1) / 2 as i32;

        let mut click_rule_radius = 0;
        for y in 0..32
        {
            for x in 0..32
            {
                let index = 32 * y + x;

                if self.click_rule_data[index] != 0
                {
                    let click_rule_offset_y = (y as i32 - click_rule_center).abs();
                    let click_rule_offset_x = (x as i32 - click_rule_center).abs();

                    let new_radius = std::cmp::max(click_rule_offset_x, click_rule_offset_y) + 1;
                    click_rule_radius = std::cmp::max(click_rule_radius, new_radius);
                }
            }
        }

        let click_rule_diameter = click_rule_radius * 2;

        let click_rule_start = std::cmp::max(0, click_rule_center - click_rule_radius + 1);
        let click_rule_end   = std::cmp::min(click_rule_start + click_rule_diameter, click_rule_size);

        let mut curr_char_bits = 0u8;
        let mut curr_char_bit_count = 0u8;
        let mut result = String::with_capacity((((click_rule_diameter * click_rule_diameter) as f32) / 6.0).ceil() as usize);
        for y in click_rule_start..click_rule_end
        {
            for x in click_rule_start..click_rule_end
            {
                if curr_char_bit_count == 6
                {
                    result.push(encode_base_64_char(curr_char_bits));

                    curr_char_bits      = 0;
                    curr_char_bit_count = 0;
                }

                let index = y * click_rule_size + x;
                if self.click_rule_data[index as usize] != 0
                {
                    curr_char_bits |= 1 << curr_char_bit_count;
                }

                curr_char_bit_count += 1;
            }
        }

        if curr_char_bit_count != 0
        {
            result.push(encode_base_64_char(curr_char_bits));
        }

        result
    }
}