0.0.5 (2019-07-30)
===================
This release includes:

* [#54](https://github.com/aheart/hearth/issues/54):
  The IP address of the server can is now available as tooltip when hovering over the username 
  in the CPU metric column.
* [#24](https://github.com/aheart/hearth/issues/24):
  There is a now a disk space indicator after the Load Average Charts.
* [e08f279](https://github.com/aheart/hearth/commit/e08f279cd435e7ac8b1366683ee0cd0aa86012f2):
  SSH session will now timeout after 5 seconds of waiting for a blocking operation.



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

