0.0.9 (2022-xx-xx)
===================
This release includes:


0.0.8 (2022-06-17)
===================
This release includes:

* [#91](https://github.com/aheart/hearth/issues/91)
  Update to Actix Web 4

0.0.7 (2020-01-31)
===================
This is a maintenance release, but it includes one bug fix:

* [98ee6d8](https://github.com/aheart/hearth/commit/98ee6d86124204ce727e0926900ba97f40840545):
  Uptime is now correct on all timeframes


0.0.6 (2019-12-31)
===================
This release includes:

* [#69](https://github.com/aheart/hearth/issues/69)
  The first charts on the top of the page now show aggregated data across all the servers.
* [#73](https://github.com/aheart/hearth/issues/73)
  The number of servers on the top of the page is now in the format "online/total". ([Thermatix](https://github.com/Thermatix))
* [#35](https://github.com/aheart/hearth/issues/35)
  On the very top of the page there are three buttons that allow switching between 3 timeframes.
* [#72](https://github.com/aheart/hearth/issues/72)
  Disk charts and Network charts are now rendered using bars instead of lines.
* [#67](https://github.com/aheart/hearth/issues/67)
  It's now possible to use pubkey authentication.


0.0.5 (2019-07-30)
===================
This release includes:

* [#54](https://github.com/aheart/hearth/issues/54):
  The IP address of the server is now available as a tooltip when hovering over the hostname 
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

