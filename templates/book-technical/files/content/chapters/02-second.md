# Keep an invariant

## Name what stays true

An [invariant](glossary:invariant) describes a fact preserved through a sequence
of steps. It can turn a loop into an argument that a program does what we claim.

> [!DEFINITION] {#def:invariant} Loop invariant
>
> An invariant holds before the first iteration and remains true after
> each iteration. It helps connect the initial state to the final result.

> [!EXAMPLE] {#example:sum} Accumulating the first integers
>
> Start with a total of zero. After processing $k$ integers, the total is
> the sum of the integers from one through $k$.

```python
def triangular(n):
    total = 0
    for k in range(1, n + 1):
        total += k
    return total
```

At termination, $\autoref{thm:sum}$ predicts the result. The code accepts
nonnegative integers; extending the empty sum to $n=0$ gives zero as well.[^domain]
The [invariant entry](index:invariant) links this discussion to the reference notes.

## Test the argument

What must change if the loop starts at two? State the initial invariant,
explain one iteration and connect the exit condition to the claimed result.

[^domain]: The function above is a teaching example, not an input-validation API.
