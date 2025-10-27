# cgns-rust

A way to read/write CGNS files from rust code.

## cgns-sys

Bindgen around the CGNS MLL ([repo](https://github.com/CGNS/CGNS), [docs](https://cgns.github.io/cgns-modern.github.io/standard/MLL/CGNS_MLL.html#standardmll)).

Issues with using the MLL (instead of CGIO):

* It is not thread-safe
* It performs transformation at file opening time (to expose the latest API on older files)
* Due to the API, when it transforms data and you then read that data, you are storing it in memory twice
* There is an issue with CGNS 3.4 files: <https://github.com/CGNS/CGNS/discussions/355#discussioncomment-4262074>

## cgns-rust

Rust wrapper around cgns-sys.
