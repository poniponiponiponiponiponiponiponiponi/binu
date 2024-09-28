# binu
binu (binary 犬) is a simple utility program for simple operations on
binary files, like grepping or search&replacing. Why? Writing with
grep `LANG=C grep -obUaP "\x00\xff"` is a joke and I can't remember
how to monkeypatch libc with patchelf either.

## running
You probably want to run it as an executable and not as a library. To
execute the program with cargo run

```sh
$ cargo run --features=build-binary -- -h
```

in the directory after doing a `git clone` and cding, to get a help
menu.
