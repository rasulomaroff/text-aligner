# text-aligner

This project has been created just for educational purposes. I'm exploring Rust and want to see its strengths and weaknesses.
This is the reason why for a relatively simple task (aligning text) so many concepts and abstractions were used.

## Usage

- First, clone the repo
- Make sure you have rust installed on your machine
- Then, run the following command

```
  cargo run <source-file> <line-width> <align> <destination-file?>
```

1. `<source-file>` - a file which contents you want to align
2. `<line-width>` - maximum width of the line allowed
3. `<align>` - align. 3 options - `left`, `right`, `justify`
4. `<destination-file>` - a file you want to put results into. Optional. If not specified, stdout will be used for this purpose.

Resulting cmd would look like this:

```
  cargo run text.txt 40 justify result.txt
```

You can also compile the binary and use its name instead of `cargo run`:

```
  text-aligner text.txt 40 justify result.txt
```
