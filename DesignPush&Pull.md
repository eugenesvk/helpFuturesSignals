Two flavors of signals: __push__, and __pull__:
  1) Push model: Essentially uses callbacks (or similar). It's quite similar to event listeners
    + never misses changes
    + good performance
    + low latency
    - sometimes has poor behavior/performance in some common situations
    - a lot of hidden performance costs in Rust (due to Rust's memory model)
    - BIG: can trigger multiple updates for a single change, see the __Diamond model__
  2) Pull model: Requires you to regularly poll the signal for changes
    + very simple to implement
    + works wonderfully with Rust's memory model
    - inefficient and has high latency (because it has to constantly poll)
  3) Hybrid push + pull: push to notify that a change has occurred, but pull to retrieve the changes
    + very efficient
    + handles every situation correctly
    * no benchmarks, but I suspect that a hybrid push + pull system will be faster than a pure push system, at least in Rust
    * possible to convert a push-based event into a push+pull Future/Stream/Signal, so even with a push+pull system you can efficiently react to external push-based events
    * Futures crate and futures-signal is based on this hybrid push+pull

__Diamond model__
Consider this hypothetical graph of signals:
  A
 / \
B   C
 \ /
  D
When __A__ changes we want to automatically update __B and C__, and when either __B or C__ changes, we want to automatically update __D__. This sort of __diamond pattern__ happens often with signals, so it's important that it behaves correctly and efficiently.
Let's suppose that __A changes__. What happens now?
  - __pull__: it's quite simple: it polls D, which then polls B and C, then B and C poll A, and everything works great.
  - __push__: A will push the change to B, which then pushes the change to D... and then A pushes the change to C, which pushes the change to D.
Notice that it pushed the change __twice to D__, which is __inefficient__ (imagine that D is some complex map which does some heavy computation, now that computation will be done twice!).


It's also important to note that all this __polling__ is an internal __implementation detail__. When a user wishes to use Futures/Streams/Signals, they're not doing any manual polling, instead they're doing something like this:
```rust
let some_signal = mutable.signal()
    .map   	(|x| x + 10)
    .filter	(|x| x < 10);
spawn_future(some_signal.for_each(|x| {
    println!("Signal value changed to: {}", x);
    ready(())
}));
```
In other words, they use handy combinators like map, filter, etc. and when they're done they use for_each + spawn to actually run the Future/Stream/Signal.
The __for_each__ method __internally polls__, but you don't need to worry about any of that, it's an implementation detail.

My Signals fully support a wide range of methods, including map, inspect, dedupe, map_future, filter_map, flatten, switch, wait_for, and, not, or, and more.

And all of the various combinators and Signals efficiently __support cancellation__.

Not only that, but the methods are __extremely fast__: as an example, map is fully stack allocated (even the closure, input Signal, and output Signal are stack allocated), and it's constant time, and the constant time is very fast.

How fast? This line is the entire implementation of map (yes, really). You might think that it's hiding all the complexity somewhere else, but it's not. If you fully expand the code, it becomes this:

```rust
match self.signal.poll_change(waker) {
  Poll::Ready(Some(value)) => Poll::Ready(Some((self.callback)(value))),
  a => a,
}
```
It's just normal pattern matching on an enum. This is about as simple and efficient as it gets.

"__lossy signal__":
  When I say that "you should think of Signals as being like mutable variables", I mean it. Let's consider a mutable variable:
```rust
let mut foo = 0;
foo = 1; // this value is ignored
foo = 2;
println!("{}", foo); // will print 2
```
Intermediate values don't matter, only the current value matters.

The same is true with my Signals library:
```rust
let foo = Mutable::new(0);
foo.set(1); // this value is ignored
foo.set(2);
// If you have some Signals which are listening to changes to foo, they will only receive the value 2, as far as they're concerned the value 1 never even existed
```
Intermediate changes are discarded and ignored, only the last one counts. Or to put it another way, Signals only care about their current value, not their past values.

It's guaranteed that you will always receive the correct current value, but you cannot rely upon receiving every value (because intermediate values might be ignored).

That has a lot of benefits in terms of performance, and also having a clean and correct API (some combinators like switch or flatten behave weirdly if you keep past values).

On the other hand, __Streams__ are an __ordered sequence of values__, and they internally use a buffer to hold values which haven't been processed yet, so if you use Streams you are guaranteed that it won't lose events, and it also guarantees that the events will occur in the correct order, which is important for events!
That means that although Signals are quite bad for events, Streams work great! It's easy to convert from an event-based system into a Stream-based system (and then you gain many useful Stream combinators).
Since my Signals are built on top of the futures crate, it has great support for converting to/from Futures and Streams, so you can mix-and-match Futures/Streams/Signals in the same program (I recommend doing this, because each of them is useful in different situations, there isn't a one-size-fits-all data structure!)

[Here is a very simplified working example of a push+pull system without cancellation](https://play.rust-lang.org/?version=stable&mode=debug&edition=2015&gist=4dbbfafb4400966a8090f003787a1086)
