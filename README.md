Grokking Rust With Data Structure Implementations
====

These are exercises inspired by Gankro's book
[Learning Rust With Entirely Too Many Linked
Lists](http://cglab.ca/~abeinges/blah/too-many-lists/book/) and were done after
looking away from the book for a few days.

* `mutable_linked_list` takes a different approach to a safe implementation of
  the mutable singly-linked list that caused me much pain and learning when
  dealing with mutable iterators.

* `immutable_linked_list` ended up being almost an exact replica.

* `mutable_doubly_linked_list` uses owned `next` pointers and unsafe `prev`
  pointers but after a subtle oversight caught by the tests, it was refactored
  so that the unsafe reverse-traversing code has almost the identical structure
  as the safe forward-traversing code.

License
====
Copyright 2015 Ashish Myles.

Distributed under the terms of both the MIT license and the Apache License
(Version 2.0) license. See [LICENSE-MIT](LICENSE-MIT) and the
[LICENSE-APACHE](LICENSE-APACHE) for details.
