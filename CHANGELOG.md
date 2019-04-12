0.0.4 (2019-04-12)
===================
This release includes:

* [#31](https://github.com/aheart/hearth/issues/31):
  SSH connection timeout is set to 1 second. This improves representation of servers which are down.
* [#19](https://github.com/aheart/hearth/issues/19):
  Each server now has a dedicated a dedicated metric buffer.
* [#46](https://github.com/aheart/hearth/issues/46):
  Ram charts now also show Cache and Buffers.
* [#48](https://github.com/aheart/hearth/pull/48):
  Memory footprint has been reduced by ~68% (when monitoring 20 servers).
* [#49](https://github.com/aheart/hearth/issues/49):
  CPU charts now display user, nice, system, idle, iowait, irq, softirq.
* [#53](https://github.com/aheart/hearth/pull/53):
  Aggregated numbers at the top of the page now also feature the number of machines being monitored.

