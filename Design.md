[comment](https://github.com/Pauan/rust-signals/issues/1)
For events I don't recommend using Signals, because __Signals are lossy__.
The purpose of Signals is to act like a __mutable variable__ which lets you be __notified__ when the __value changes__. So if your use case fits into that mental model, then Signals work great!

But if instead you want an __ordered sequence of multiple values__, a __Stream__ is much better (Streams are provided by the futures crate). They maintain an internal queue so that you don't lose any events.


[source](discord.com/channels/716581000800632853/716581000800632856/968318774207971329)

Sycamore (another Rust Wasm framework) is closer to SolidJS
the API is indeed completely different, however the idea of fine-grained FRP updates is the same
__dominator__ takes the Rust approach: the highest performance possible, and everything is explicit
__SolidJS__ takes the JS approach: it's lower performance, and it has a lot of magic (things are hidden and done automatically for you)
with dominator you always know exactly what is changing, and where, and when, and why, nothing is hidden from you
this gives you complete control over both the behavior and the performance
futures-signals also emphasizes the concept of first-class FRP values
whereas something like SolidJS tries to make everything magic and automatic, so you can pretend that FRP doesn't exist and everything "just happens"
with dominator you can't pretend that FRP doesn't exist, it's the backbone of everything


for example... something that is trivial in dominator: take a Signal as input, map it, filter it, run an asynchronous Future, convert it into a Stream, convert it back into a Signal, dedupe its value, debounce it, delay it by 500ms
since everything is first-class, you have complete control over the value, the updating, and time
this is how FRP was originally envisioned to be
since it's treating mutation (and time) as first-class values, it gives you a lot of precision and control
and yes, almost all FRP implementations are bad, they're slow, and they're buggy
I would know, since I've created many slow and buggy FRP implementations over the years
futures-signals fixes a ton of the bugs that other FRP implementations have, as far as I know it's bug free

for example, the __diamond problem__ doesn't exist
and the __latency problem__ of poll-based Signals doesn't exist either
also the __hot/cold problem__ doesn't exist
and __Elm's foldp problems__ don't exist either (that's a big one, since it's fundamental to the very design of FRP)

in particular the foldp memory leaks
so futures-signals has dynamically-updating signal graphs without any issues
because it's based on a more rigorously correct view of signals
other FRP implementations try really really hard to combine Signals and Streams together into 1 type, but that's a fundamental mistake
futures-signals keeps them as two separate types, which fixes all the bugs, memory leaks, and time leaks
mikayla — 04/26/2022



__Sycamore__: also using FRP, though its implementation is __completely different__ from dominator
  + __create_effect__ is pretty cool, it basically replaces a lot of combinators
  - uses even more magic than dominator does though, not sure how I feel about that
  - implementation of FRP seems pretty inefficient, but I think it would be possible to optimize it
I feel like it should be possible to abuse Deref to make it more ergonomic so you could do something like this:
```rust
let s1 = mutable1.signal();
let s2 = mutable2.signal();
let s3 = map!(*s1 + *s2);
// instead of this:
let s3 = map_ref! {
    let s1 = mutable1.signal(),
    let s2 = mutable2.signal() =>
    *s1 + *s2
};
```
but some concessions would have to be made


there's a bunch of FRP stuff in JS, such as RxJS, though RxJS barely counts as FRP in my opinion: it's much closer to an event system
but as an example of an FRP-ish system, look into __mobx__, and especially __mobx-react__, which basically does the same thing as __dominator + future-signals__ except... a lot worse in my opinion


(I'm sure dominator scales exceptionally well)
(its raw performance is one of the best out of all frameworks, and because of the way FRP works the performance is O(1) no matter how big or deeply nested your app is)
for code size dominator has a large upfront cost (because it needs the Rust stdlib and wasm glue code), but it scales very well, because a Rust function is smaller than the equivalent JS function
so small apps are bigger with dominator, but large apps are smaller
even a large dominator app should only be ~500 kB or so
and because wasm can compile while streaming the bytes, you get a much faster time-to-first-paint even if the wasm file is larger than the JS f ile


pub fn main_js() {
    stylesheet!(...

