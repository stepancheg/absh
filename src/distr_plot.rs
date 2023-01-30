use crate::bars::plot_halves_u64;
use crate::bars::plot_u64;
use crate::experiment::Experiment;
use crate::experiment_map::ExperimentMap;
use crate::math::number::Number;
use crate::math::numbers::Numbers;

pub(crate) fn make_distr_plots<N: Number>(
    tests: &ExperimentMap<Experiment>,
    width: usize,
    numbers: impl Fn(&Experiment) -> &Numbers<N>,
) -> anyhow::Result<ExperimentMap<String>> {
    let min = tests
        .values()
        .map(|t| numbers(t).min().unwrap())
        .min()
        .unwrap();
    let max = tests
        .values()
        .map(|t| numbers(t).max().unwrap())
        .max()
        .unwrap();

    let distr_halves: ExperimentMap<_> =
        tests.map(|t| (t, numbers(t).distr(width * 2, min.clone(), max.clone())));

    let distr: ExperimentMap<_> =
        tests.map(|t| (t, numbers(t).distr(width, min.clone(), max.clone())));

    let max_height_halves = distr_halves
        .values()
        .map(|(_, d)| d.max())
        .max()
        .unwrap()
        .clone();
    let max_height = distr.values().map(|(_, d)| d.max()).max().unwrap().clone();

    let distr_plots = distr.map(|(t, d)| plot_u64(&d.counts, max_height, &t.plot_highlights()));

    let distr_halves_plots = distr_halves
        .map(|(t, d)| plot_halves_u64(&d.counts, max_height_halves, &t.plot_halves_highlights()));

    if max_height_halves <= 2 {
        Ok(distr_halves_plots)
    } else {
        Ok(distr_plots)
    }
}
