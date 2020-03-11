<?hh

use namespace HH\Lib\OS;
use namespace HH\Lib\_Private\_OS;

<<__EntryPoint>>
async function main(): Awaitable<void> {
  list($r, $w) = _OS\pipe();

  var_dump(await _OS\poll_async($r, \STREAM_AWAIT_READ, 1));
  _OS\write($w, "Helo, world!\n");
  var_dump(await _OS\poll_async($r, \STREAM_AWAIT_READ, 100));
  // _OS_\read not implemented yet
}
