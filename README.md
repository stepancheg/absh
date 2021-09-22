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
A: n=210 mean=1.367 std=0.109 se=0.007 min=1.222 max=2.403 med=1.353
B: n=210 mean=1.321 std=0.081 se=0.005 min=1.182 max=1.706 med=1.305
A: distr=[  ▁▂▄▅▆▄▅▆▅▃▂▁    ▁                                      ]
B: distr=[ ▁▃▅▄█▆▄▄▂▂▂  ▁                                          ]
B/A: 0.967 0.953..0.980 (95% conf)
```
