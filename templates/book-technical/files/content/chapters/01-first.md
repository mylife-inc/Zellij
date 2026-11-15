# Start with a sum

## Notice the structure

Consider the [finite sum](glossary:sum) of the first $n$ positive integers.
Pair the first term with the last, then the second with the next-to-last.
This observation suggests an identity, recorded in the sample notes [@sample-note].

$$\sum_{k=1}^{n} k = \frac{n(n+1)}{2}\label{eq:sum}$$

> [!THEOREM] {#thm:sum} A finite arithmetic sum
>
> Equation $\eqref{eq:sum}$ holds for every positive integer $n$.

> [!PROOF] {#proof:sum} Pairing the terms
>
> Write the sum once in increasing order and once in decreasing order.
> Every column totals $n+1$, and there are $n$ columns. Thus twice the
> sum is $n(n+1)$; divide by two to obtain $\autoref{thm:sum}$.

## Check a small case

| Terms | Direct addition | Formula |
| --- | --- | --- |
| 3 | 1 + 2 + 3 = 6 | 3 × 4 / 2 = 6 |
| 5 | 1 + 2 + 3 + 4 + 5 = 15 | 5 × 6 / 2 = 15 |

These examples help detect mistakes; they do not replace the proof.
The [subject index](index:sum) collects every marked discussion.

## Try it yourself

Explain why pairing also works when $n$ is odd. Then use the same idea to add
an arithmetic sequence with a different first term and a fixed common difference.
