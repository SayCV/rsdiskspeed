# rsdiskspeed

Disk speed test for Rust Inspired from [MonkeyTest](https://github.com/thodnev/MonkeyTest), test your hard drive read-write speed.

## Compiling from Source

```
git clone https://github.com/SayCV/rsdiskspeed
cd rsdiskspeed
cargo build --release
```

## How to use

```

mkdir udisk
mount /dev/sdb1 udisk
./rsdiskspeed -f udisk/rsdspdtest

```
