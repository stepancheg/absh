use std::f64;

fn bar_char_0_8(value: u32) -> Option<char> {
    Some(match value {
        0 => ' ',
        1 => '▁',
        2 => '▂',
        3 => '▃',
        4 => '▄',
        5 => '▅',
        6 => '▆',
        7 => '▇',
        8 => '█',
        _ => return None,
    })
}

pub fn bar_char_for_range(value: f64, min: f64, max: f64) -> Option<char> {
    bar_char_0_8(((value - min) / (max - min) * 8.0).round().clamp(0.0, 8.0) as u32)
}

pub fn bar_char_1_for_range(value: f64, min: f64, max: f64) -> Option<char> {
    bar_char_0_8(
        ((value - min) / (max - min) * 7.0 + 1.0)
            .round()
            .clamp(1.0, 8.0) as u32,
    )
}

pub fn plot(values: &[f64], min: f64, max: f64) -> String {
    values
        .iter()
        .map(|v| bar_char_for_range(*v, min, max).unwrap_or('X'))
        .collect()
}
