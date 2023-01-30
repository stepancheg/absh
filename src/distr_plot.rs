use crate::bars::plot_halves_u64;
use crate::bars::plot_u64;
use crate::experiment::Experiment;
use crate::math::number::Number;
use crate::math::numbers::Numbers;

pub(crate) fn make_distr_plots<N: Number>(
    tests: &[Experiment],
    width: usize,
    numbers: impl Fn(&Experiment) -> &Numbers<N>,
) -> anyhow::Result<Vec<String>> {
    let min = tests
        .iter()
        .map(|t| numbers(t).min().unwrap())
        .min()
        .unwrap();
    let max = tests
        .iter()
        .map(|t| numbers(t).max().unwrap())
        .max()
        .unwrap();

    let distr_halves: Vec<_> = tests
        .iter()
        .map(|t| (t, numbers(t).distr(width * 2, min.clone(), max.clone())))
        .collect();

    let distr: Vec<_> = tests
        .iter()
        .map(|t| (t, numbers(t).distr(width, min.clone(), max.clone())))
        .collect();

    let max_height_halves = distr_halves
        .iter()
        .map(|(_, d)| d.max())
        .max()
        .unwrap()
        .clone();
    let max_height = distr.iter().map(|(_, d)| d.max()).max().unwrap().clone();

    let distr_plots = distr
        .iter()
        .map(|(t, d)| plot_u64(&d.counts, max_height, &t.plot_highlights()))
        .collect();

    let distr_halves_plots = distr_halves
        .iter()
        .map(|(t, d)| plot_halves_u64(&d.counts, max_height_halves, &t.plot_halves_highlights()))
        .collect();

    if max_height_halves <= 2 {
        Ok(distr_halves_plots)
    } else {
        Ok(distr_plots)
    }
}
