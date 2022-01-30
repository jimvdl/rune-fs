# rune-fs

[![Build](https://github.com/jimvdl/rune-fs/workflows/build/badge.svg)](https://github.com/jimvdl/rune-fs)
[![API](https://docs.rs/rune-fs/badge.svg)](https://docs.rs/rune-fs)
[![Crate](https://img.shields.io/crates/v/rune-fs)](https://crates.io/crates/rune-fs)
[![dependency status](https://deps.rs/repo/github/jimvdl/rune-fs/status.svg)](https://deps.rs/repo/github/jimvdl/rune-fs)

Read-only, low level, virtual file types for the RuneScape file system.

This crate supplies all of the backing types for [rs-cache](https://docs.rs/rs-cache). Many of these
types were private but are now publicly available. rs-cache is a high level api for both the OSRS and RS3 
caches and exposing these low level virtual types didn't make sense, hence this crate.

A word of caution, these types are highly experimental, I have done my best to document and test as
much as I can, but there might still be the weird occasional edge-case. With that said, whenever you find
a bug or missing feature; or even unsoundness don't hesitate to 
[open an issue](https://github.com/jimvdl/rs-cache/issues/new).

Useful links:\
&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;<img src="https://oldschool.runescape.wiki/images/thumb/5/5d/Fire_rune_detail.png/800px-Fire_rune_detail.png?07ed5" width="10"> &nbsp;[Releases](https://github.com/jimvdl/rs-cache/tags)\
&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;<img src="https://oldschool.runescape.wiki/images/thumb/7/74/Water_rune_detail.png/800px-Water_rune_detail.png?4e790" width="10"> &nbsp;[Documentation](https://docs.rs/rune-fs)