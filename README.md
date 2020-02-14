# stork.rs

 [![License: WTFPL](https://img.shields.io/badge/License-WTFPL-brightgreen.svg?style=flat-square&logo=appveyor)](http://www.wtfpl.net/about/) [![Docs](https://docs.rs/stork/badge.svg)](https://docs.rs/stork/) [![Downloads](https://img.shields.io/crates/d/stork.svg?style=flat-square&logo=appveyor)](https://crates.io/crates/stork)

`stork` is a simple futures-based library to recursively crawl
sources in a search engine-like fashion. stork was designed from the
ground to have a simple API that is easy to use and can be reused
across multiple protocols, yielding each result giving end users the
freedom to do BFS, DFS or any type of search they may so wish.

**The API is extremely unstable currently and will likely go through quite a few revisions before we get it stable, I'll keep on top of the changelogs but please keep this in mind when using the library.**

>i am a heron. i haev a long neck and i pick fish out of the water w/ my beak. if you dont star this repo and 10 other rust repos u enjoy i will fly into your kitchen tonight and make a mess of your pots and pans


View the docs for examples of how to use `stork`:
- [stork](https://docs.rs/stork/)
- [stork_http](https://docs.rs/stork_http/)

or look in the [examples/](https://github.com/w4/stork/tree/master/examples) directory for some real-world examples!

## storkcli

`storkcli` is built off the back of stork. It can be used to scrape websites for links using various
filters, though basic right now `stork` gives us the ability to make this CLI as sophisticated as we like.

Usage:

```
Usage: ./storkcli <url> [--max-depth <max-depth>]

Link hunter with a little bit of magic.

Options:
  --max-depth       specifies how deep we should go from the origin, leave this
                    value unspecified to recurse until there's nothing left to
                    follow.
  --help            display usage information
```

Example:

```
$ ./storkcli "https://doyle.la/" --max-depth 0
↳ https://instagram.com/doyl_e
↳ https://linkedin.com/in/jordanjdoyle
↳ https://stackoverflow.com/users/2132800/jordan-doyle
↳ https://last.fm/user/doyle-
↳ https://github.com/w4
↳ mailto:jordan@doyle.la
↳ https://keybase.io/jrd
```
