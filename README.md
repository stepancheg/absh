# absh: A/B testing for shell scripts

```
$ absh \
  -a "test a" \
  -b "test b" \
  -A "warmup for a" \
  -B "warmup for b"
```

It continuously run `B`, `b`, `A`, `a`; ignores the numbers of the first iteration,
and then after third iteration it prints average and standard deviation:

```
A: N=12, r=65.748+-1.704
B: N=12, r=67.983+-2.440
B/A: 1.034
```
