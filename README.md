# Zeta

A library to stream the zeros from the Riemann zeta function with high precision at blasting speed.

The data is sourced from the [LMFDB](https://www.lmfdb.org/).
They serve an [open dataset](https://beta.lmfdb.org/riemann-zeta-zeros/) from where the zeta zeros can be extracted.

All zeros will be parsed and streamed to a sink provided to the crate.

## Credit

Credit where credit is due. Thanks to the people at LMFDB for providing this high precision data set.
Also thanks to Jonathan Bober who wrote the
[python script](https://beta.lmfdb.org/riemann-zeta-zeros/examples/every_millionth_zero/platt_zeros.py) where this
crate is reverse engineered from.

## Motivation

I'm a nerd with passions. I know there are other libraries that already provide the zeta zeros without all this
overhead. I still wanted to create this out of love for the LMFDB project and the Riemann Hypothesis.

## Usage

First, create an instance of the `zeta::ZeroStream` trait. The trait itself has two methods. Method `is_closed` is a
predicate that tells the crate when it should stop streaming zeros. Method `send` is called by the crate whenever a zero
is found.

The [example](./examples/first_5000.rs) has a minimal implementation of a `ZeroStream` that's configured to fetch the
first 5000 zeros in the dataset.

Then configure a `zeta::SeekPattern` to tell the stream how much zeros it should fetch.

Pass both the `ZeroStream` and the `SeekPattern` to the `zeta::zero_stream` method.

# Drawbacks

Part of the dataset is a sqlite database that holds the location of all data block in the `.dat` file of the dataset.
By default, the crate will stream the database using sqlite HTTP VFS. This process can be quite slow
(1.2 GB, up to 4 minutes), so as an alternative you can download the database locally and refer to it using an
environment variable.

```bash
export RUST_LOG=debug
cargo run --example first_5000
# - This can take up to 4 minutes.
```

```bash
export RUST_LOG=debug
export ZETA_DB="$HOME/index.db"
cargo run --example first_5000
# - This will execute immediately.
```

Future optimizations include that `SeekPattern::None` will not require database access and thus will always run
smoothly.

## Correctness of the Data

From the output of the example, samples were takes and manually compared to samples takes from the
[LMFDB Zeros of Zeta page](https://www.lmfdb.org/zeros/zeta/).

Although zeta's output matches that of LMFDB, we noticed we had some more precision.
The precision on LMFDB seems to be a rounded version of zeta's output.

Finding out if this extra precision is correct, is a challenge for another time.

```
Zeto number 2304161
-------------------
Zeta:  1289004.7448121853581234567819561293179851
LMFDB: 1289004.7448121853581234567819561293180
                                             ^^^^

Zero number 236207
------------------
Zeta:  162060.6101577826246269167329812345679401
LMFDB: 162060.6101577826246269167329812345679
                                            ^^^^
```

## Documentation of the Dataset

The [.dat files](https://beta.lmfdb.org/riemann-zeta-zeros/data/) all start with a file header. Then they are
subdivided in blocks of zeroes.

Let's take [zeros_5000.dat](https://beta.lmfdb.org/riemann-zeta-zeros/data/zeros_5000.dat) as example.

**File Header** | Each file starts with a file header of 8 bytes long.

These bytes, interpreted as a little endian
unsigned 64-bit integer, represent the number of blocks in this file. For zeros_5000.dat this value is 10.

**Block** | For each block we first move to the `offset` location in the file. Then we read the block header.

The block header consists of 32 bytes interpreted as:

1. `t0`: A 64-bit LE floating point: Same as the `t` column in the database.
2. `t1`: A 64-bit LE floating point: Same as the `t` column but for the next block.
3. `Nt0`: A 64-bit LE unsigned integer: The same as the `N` column in the database.
4. `Nt1`: A 64-bit LE unsigned integer: The same as the `N` column but for the next block.

If we read the block header for block 0 in zeros_5000.dat we get these values respectively.

1. 5000.0
2. 7100.0
3. 4520
4. 6814

**Zeros** | From this point on we start reading zeros. The data we actually read still needs to be processed before it
actually becomes a zero.

If you subtract `Nt0` from `Nt1` you'll get the number of zeros in this block.

After you've read the block header, for the amount of zeros in this block, read 13 bytes. Convert those bytes to:

1. `z1`: A 64-bit unsigned integer.
2. `z2`: A 32-bit unsigned integer.
3. `z3`: An 8-bit unsigned integer.

Now we bitshift and add these numbers.

`z = (z3 << 96) + (z2 << 64) + z1`

The first zero of this block equals to:

$$t_0 + z * 2^{-101}$$

In order to find the next zero, decode the next 13-byte block entry, add it to `z` and recalculate the formula above.

After reading the first block, you can read the second block header to parse all zeros of that block and so on.

## Documentation of the Database

> The database is not needed if you just want to stream all zeros starting from the beginning.
>
> The database is used when you need to find the first n zeros starting from value x or the first n zeros starting
> from zero number x.

Use the database to find the file and block from where you need to start your search.
From there on out, use the method described above to extract the zeros?

`SELECT * FROM zero_index WHERE filename = 'zeros_5000.dat'`

| t       | N     | filename       | offset | block_number |
|---------|-------|----------------|--------|--------------|
| 5000.0  | 4520  | zeros_5000.dat | 8      | 0            |
| 7100.0  | 6814  | zeros_5000.dat | 29862  | 1            |
| 9200.0  | 9209  | zeros_5000.dat | 61029  | 2            |
| 11300.0 | 11681 | zeros_5000.dat | 93197  | 3            |
| 13400.0 | 14215 | zeros_5000.dat | 126171 | 4            |
| 15500.0 | 16802 | zeros_5000.dat | 159834 | 5            |
| 17600.0 | 19435 | zeros_5000.dat | 194095 | 6            |
| 19700.0 | 22106 | zeros_5000.dat | 228850 | 7            |
| 21800.0 | 24815 | zeros_5000.dat | 264099 | 8            |
| 23900.0 | 27555 | zeros_5000.dat | 299751 | 9            |

The database has ten entries. This value will always correspond to the number of blocks in a file header.

- `t`: The values that will be decoded from this block will cumulatively be added to the value of `t` in order to get
  the next zero. This is also the ID column of the table. Can be used to find zeros starting from value x.
- `N`: The first zero decoded from this block will be the `N`'th zero. Can be used to find zeros starting from zero
  number x.
- `filename`: The file where this block is located.
- `offset`: The first byte of this block in the file.
- `block_number`: Sequence number of this block.

## About the Author

I broke my arm on May 1st 2025. The two weeks after that I had a lot of time and a lot of painkillers. I'd have loved to
continue on my other projects. Due to all distractions it seemed safer to pause them for now and temporarily work on
something small, something new.

I still yearned for some rust and so far I hadn't done any async projects.
LMFDB had also intrigued me for a long time.
No better time to do this then now, I thought.

This whole project is written with one hand (go lefties!) and dosed on painkillers.

Enough boasting for now. I know there are other libraries that already provide the zeta zeros without all this overhead.
I still wanted to create this out of love for the LMFDB project and the Riemann Hypothesis.