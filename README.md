# absh: A/B testing for shell scripts

```
$ absh \
  -a "test a" \
  -b "test b" \
  -A "warmup for a" \
  -B "warmup for b"
```

It continuously run `B`, `b`, `A`, `a`; ignores the numbers of the first iteration,
and then after third iteration it prints averages, and 95% confidence interval of B average/A average.

```
A: n=30 r=14.388+-0.959 se=0.178
B: n=30 r=12.682+-1.221 se=0.226
B/A: 0.845..0.919 (95% conf)
```
