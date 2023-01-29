use std::fmt::Write;

use crate::ansi;
use crate::math::number::Number;
use crate::math::numbers::Numbers;
use crate::measure::MeasureDyn;
use crate::student::t_table;
use crate::student::TWO_SIDED_95;
use crate::test::Test;

pub(crate) fn render_stats<N: Number>(
    tests: &[Test],
    include_distr: bool,
    measure: &dyn MeasureDyn,
    numbers: impl Fn(&Test) -> &Numbers<N>,
) -> anyhow::Result<String> {
    let mut r = String::new();

    let stats: Vec<_> = tests.iter().map(|t| numbers(t).stats()).collect();

    let stats_str: Vec<_> = measure.display_stats(tests);

    let stats_width = stats_str.iter().map(|s| s.len()).max().unwrap();

    let distr_plots = measure.make_distr_plots(&tests, stats_width - 8)?;

    writeln!(r, "{}:", measure.name())?;
    for index in 0..tests.len() {
        let test = &tests[index];
        let stats = &stats_str[index];
        writeln!(
            r,
            "{color}{name}{reset}: {stats}",
            name = test.name,
            color = test.name.color(),
            reset = ansi::RESET,
        )?;
    }
    for index in 0..tests.len() {
        let test = &tests[index];
        let plot = &distr_plots[index];
        if include_distr {
            writeln!(
                r,
                "{color}{name}{reset}: distr=[{plot}]",
                name = test.name,
                color = test.name.color(),
                reset = ansi::RESET,
            )?;
        }
    }

    if tests.len() >= 2 {
        for b_index in 1..tests.len() {
            let degrees_of_freedom =
                u64::min(stats[0].count as u64 - 1, stats[b_index].count as u64 - 1);
            let t_star = t_table(degrees_of_freedom, TWO_SIDED_95);

            // Half of a confidence interval
            let conf_h = t_star
                * f64::sqrt(
                    stats[0].sigma_sq() / (stats[0].count - 1) as f64
                        + stats[b_index].sigma_sq() / (stats[b_index].count - 1) as f64,
                );

            // Quarter of a confidence interval
            let conf_q = conf_h / 2.0;

            let b_a_min =
                (stats[b_index].mean.as_f64() - conf_q) / (stats[0].mean.as_f64() + conf_q);
            let b_a_max =
                (stats[b_index].mean.as_f64() + conf_q) / (stats[0].mean.as_f64() - conf_q);

            writeln!(
                r,
                "{b_name}/{a_name}: {b_a:.3} {b_a_min:.3}..{b_a_max:.3} (95% conf)",
                b_name = tests[b_index].name,
                a_name = tests[0].name,
                b_a = stats[b_index].mean.as_f64() / stats[0].mean.as_f64(),
                b_a_min = b_a_min,
                b_a_max = b_a_max,
            )?;
        }
    }

    Ok(r)
}
