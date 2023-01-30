use std::fmt::Write;

use crate::ansi;
use crate::experiment::Experiment;
use crate::experiment_map::ExperimentMap;
use crate::math::number::Number;
use crate::math::numbers::Numbers;
use crate::measure::tr::MeasureDyn;
use crate::student::t_table;
use crate::student::TWO_SIDED_95;

pub(crate) fn render_stats(
    tests: &ExperimentMap<Experiment>,
    include_distr: bool,
    measure: &dyn MeasureDyn,
    numbers: impl Fn(&Experiment) -> &Numbers,
) -> anyhow::Result<String> {
    let mut r = String::new();

    let stats: ExperimentMap<_> = tests.map(|t| numbers(t).stats().unwrap());

    let stats_str: ExperimentMap<String> = measure.display_stats(tests);

    let stats_width = stats_str.values().map(|s| s.len()).max().unwrap();

    let distr_plots = measure.make_distr_plots(&tests, stats_width - 8)?;

    writeln!(r, "{}:", measure.name())?;
    for (_name, test, stats) in tests.zip(&stats_str) {
        writeln!(
            r,
            "{color}{name}{reset}: {stats}",
            name = test.name,
            color = test.name.color(),
            reset = ansi::RESET,
        )?;
    }
    for (_name, test, plot) in tests.zip(&distr_plots) {
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

    let mut stats_iter = stats.iter();
    let (a_name, stats_a) = stats_iter.next().unwrap();
    for (b_name, stats_b) in stats_iter {
        let degrees_of_freedom = u64::min(stats_a.count as u64 - 1, stats_b.count as u64 - 1);
        let t_star = t_table(degrees_of_freedom, TWO_SIDED_95);

        // Half of a confidence interval
        let conf_h = t_star
            * f64::sqrt(
                stats_a.sigma_sq() / (stats_a.count - 1) as f64
                    + stats_b.sigma_sq() / (stats_b.count - 1) as f64,
            );

        // Quarter of a confidence interval
        let conf_q = conf_h / 2.0;

        let b_a_min = (stats_b.mean.as_f64() - conf_q) / (stats_a.mean.as_f64() + conf_q);
        let b_a_max = (stats_b.mean.as_f64() + conf_q) / (stats_a.mean.as_f64() - conf_q);

        writeln!(
            r,
            "{b_name}/{a_name}: {b_a:.3} {b_a_min:.3}..{b_a_max:.3} (95% conf)",
            b_a = stats_b.mean.as_f64() / stats_a.mean.as_f64(),
            b_a_min = b_a_min,
            b_a_max = b_a_max,
        )?;
    }

    Ok(r)
}
