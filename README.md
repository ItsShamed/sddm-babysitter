# SDDM Babysitter

This is a daemon to watch for the SDDM helper and work around issue
[sddm/sddm#1908] while the maintainers are slacking their asses off.

[sddm/sddm#1908]: https://github.com/sddm/sddm/issues/1908

## How it works

The daemon basically watches for the `sddm-helper` process (if any), and waits
on it. If it exits tragically (non-zero return code), it will try to kill the
X11 process attached to SDDM, to make it act a little bit instead of stalling
around and crying like a little baby "waah! waah! my helper died non-zero code
I can't do anything! waaaaaah!!".

## Was Rust really necessary for this? It's only 90 LOC……

No, but it's funsises, and I'm a little バカ and too lazy to do proper C stuff.

> [!CAUTION]
> I absolutely have no fucking idea of what I am doing. In its current state,
> nothing is working and I'm still figuring out how to jiggle with `ptrace(2)`.
