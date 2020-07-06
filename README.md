# actix-template

Simple RESTFUL server to process different optional params.

### Expected input example

```{"a":true,"b":true, "c": true, "d": 3.7 "e": 5, "f": 2, "case": "C1"}```

*Any parameter is omittable, but due to the requirements it will result to incorrect request error.

### Task description:
RESTful API receiving parameters:
A: bool
B: bool
C: bool
D: float
E: int
F: int

### Expected output

`{h: M|P|T, k: float}`

The assignment consists of base expressions set and two custom set of
expressions that override / extend the base rules.

Base

    A && B && !C => H = M
    A && B && C => H = P
    !A && B && C => H = T
    [other] => [error]

    H = M => K = D + (D * E / 10)
    H = P => K = D + (D * (E - F) / 25.5)
    H = T => K = D - (D * F / 30)

Custom 1

    H = P => K = 2 * D + (D * E / 100)

Custom 2

    A && B && !C => H = T
    A && !B && C => H = M
    H = M => K = F + D + (D * E / 100)


## Run:

``` RUST_LOG=info cargo run```

## Test:

``` curl -H "Content-Type: application/json" -X POST -d '{"a":true,"b":true, "c": true, "d": 4.7, "e": 5, "f": 2, "case": "C1"}' localhost:3030/compute ```

### Web framework of choice:
Actix has testing utilities included so it is a convenient choice.
(warp claims itself *right* web framework, but albeit nice trace it just too ubiquitous and unclear in terms of testing)

### Error handling
Error handling made with anyhow(parsing) + actix_error(web) crates.

### Tests
Tests feature main possibles scenarios, but not all combinations of params tested, of course.
Most incorrect scenarios will be processed in either

