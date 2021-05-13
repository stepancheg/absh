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

fn bar_char_0_2_0_2(values: [u32; 2]) -> Option<char> {
    Some(match values {
        [0, 0] => ' ',
        [0, 1] => '▗',
        [0, 2] => '▐',
        [1, 0] => '▖',
        [1, 1] => '▄',
        [1, 2] => '▟',
        [2, 0] => '▌',
        [2, 1] => '▙',
        [2, 2] => '█',
        _ => return None,
    })
}

const NAN_CHAR: char = '?';

fn f64_to_bucket(
    value: f64,
    min: f64,
    max: f64,
    first_bucket: u32,
    last_bucket: u32,
) -> Option<u32> {
    assert!(first_bucket <= last_bucket);
    let value_0_1 = (value - min) / (max - min);
    let n_buckets = last_bucket + 1 - first_bucket;
    let rounded = (value_0_1 * n_buckets as f64 + first_bucket as f64).floor();
    if rounded.is_nan() {
        None
    } else {
        Some(
            rounded
                .floor()
                .clamp(first_bucket as f64, last_bucket as f64) as u32,
        )
    }
}

fn bar_char_0_8_range(value: f64, min: f64, max: f64) -> char {
    match f64_to_bucket(value, min, max, 0, 8) {
        Some(v) => bar_char_0_8(v).unwrap(),
        None => NAN_CHAR,
    }
}

fn bar_char_0_2_0_2_range(values: [f64; 2], min: f64, max: f64) -> char {
    let v0 = f64_to_bucket(values[0], min, max, 0, 2);
    let v1 = f64_to_bucket(values[1], min, max, 0, 2);
    match (v0, v1) {
        (Some(v0), Some(v1)) => bar_char_0_2_0_2([v0, v1]).unwrap(),
        _ => NAN_CHAR,
    }
}

pub fn plot(values: &[f64], min: f64, max: f64) -> String {
    values
        .iter()
        .map(|v| bar_char_0_8_range(*v, min, max))
        .collect()
}

pub fn plot_halves(values: &[f64], min: f64, max: f64) -> String {
    let mut s = String::new();
    for chunk in values.chunks(2) {
        if chunk.len() == 2 {
            s.push(bar_char_0_2_0_2_range([chunk[0], chunk[1]], min, max));
        } else {
            s.push(bar_char_0_2_0_2_range([chunk[0], min], min, max));
        }
    }
    s
}

#[cfg(test)]
mod test {
    use crate::bars::f64_to_bucket;
    use crate::bars::plot_halves;
    use crate::plot;

    #[test]
    fn test_f64_to_range() {
        assert_eq!(Some(7), f64_to_bucket(-9., 3.0, 6.0, 7, 9));
        assert_eq!(Some(7), f64_to_bucket(2.9, 3.0, 6.0, 7, 9));
        assert_eq!(Some(7), f64_to_bucket(3.0, 3.0, 6.0, 7, 9));
        assert_eq!(Some(7), f64_to_bucket(3.9, 3.0, 6.0, 7, 9));
        assert_eq!(Some(8), f64_to_bucket(4.1, 3.0, 6.0, 7, 9));
        assert_eq!(Some(8), f64_to_bucket(4.9, 3.0, 6.0, 7, 9));
        assert_eq!(Some(9), f64_to_bucket(5.1, 3.0, 6.0, 7, 9));
        assert_eq!(Some(9), f64_to_bucket(5.9, 3.0, 6.0, 7, 9));
        assert_eq!(Some(9), f64_to_bucket(6.1, 3.0, 6.0, 7, 9));
        assert_eq!(Some(9), f64_to_bucket(99., 3.0, 6.0, 7, 9));
    }

    #[test]
    fn test_plot() {
        assert_eq!(
            "   ▁▁▂▃▄▅▆▇███",
            plot(
                &[0.0, 0.1, 0.9, 1.1, 1.9, 2.1, 3.1, 4.1, 5.1, 6.1, 7.1, 8.1, 8.9, 9.0],
                0.0,
                9.0
            )
        );
    }

    #[test]
    fn test_plot_0_2() {
        assert_eq!(
            " ▟█",
            plot_halves(&[3.0, 3.9, 4.1, 5.1, 5.1, 6.0], 3.0, 6.0)
        );
    }
}
