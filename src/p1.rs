use futures_signals::signal::Mutable;
use futures_signals::signal::SignalExt; // for Iterator trait (gives for_each)
use futures::executor;

pub fn mutable() { // 1. Mutable is very similar to Arc + RwLock:
  // Implements Send & Sync , so it can be sent & used between multiple threads
  // Implements Clone       , which will create a new reference to the same Mutable (just like Arc)
  // You can retrieve the current value
  // You can change   the current value

  let my_state = Mutable::new(5);

  let mut lock = my_state.lock_mut(); // Acquires a mutable lock on my_state
  assert_eq!(*lock,  5);  println!("@mutable: *lock = {}", *lock);
  *lock = 10; // Changes the current value of my_state to 10
  assert_eq!(*lock, 10);  println!("@mutable: *lock = {}", *lock);
}

fn print_type_of<T>(_: &T) { println!("{}", std::any::type_name::<T>()) }
pub fn for_each() { // 2. for_each. Mutable vs RwLock: possible to be efficiently notified whenever the Mutable changes
    // 1. Returns a Future
    // 2. When that Future is spawned, it will call the |value| { ... } closure with the current value of my_state (which in this case is 10)
    // 3. Whenever my_state changes,   it will call the closure again with the new value
      // => can be used for automatically updating your GUI or database whenever the Signal changes
      // Just like Future/Stream, when you create a Signal it does not actually do anything until it is spawned

  let my_state = Mutable::new(5);

  // lock for some reason doesn't let the further block_on even run
  // using block_on doesn't really work with Signals, because for_each will keep waiting forever (until the Mutable is dropped) (discord.com/channels/716581000800632853/716581000800632856/852292741915475989)
    // resolves? block_on(value.first().to_future())
  // let mut lock = my_state.lock_mut();
  // *lock = 10; // Changes the current value of my_state to 10

  let pool   = executor::ThreadPool::new().expect("Failed to build pool");
  let future1 = my_state.signal().for_each(|value| { // ← is run for the current value of my_state, and also every time my_state changes
    println!("@for_each::my_state.signal().for_each value={}", value);
    async {}
  });
  let future2 = my_state.signal().for_each(|v| { println!("@f2 v={}", v); async {} });
  let future3 = my_state.signal().for_each(|v| { println!("@f3 v={}", v); async {} });
  let future4 = my_state.signal().for_each(|v| { println!("@f4 v={}", v); async {} });
  let future5 = my_state.signal().for_each(|v| { println!("@f5 v={}", v); async {} });
  let future6 = my_state.signal().for_each(|v| { println!("@f6 v={}", v); async {} });
  let future7 = my_state.signal().for_each(|v| { println!("@f7 v={}", v); async {} });

  // Ways to spawn a Future: block_on(F), tokio::spawn(F), task::spawn(F), spawn_local(F), etc.
  // print_type_of(&future);

  // 1. using futures LocalPool to run Single-Threaded
    use futures::executor::LocalPool;
    use futures::task::LocalSpawnExt;
    let spawner = LocalPool::new().spawner();
    // denotes that the async ↓ block captures ownership of the variables they close over
    spawner.spawn_local(async move { future1.await;}); // doesn't print, guess need to handle result from this
  // 2. using futures ThreadPool to run multi-threaded (also with tokio and async-std)
    use futures::executor::ThreadPool;
    let pool = ThreadPool::new().unwrap();
    pool.spawn_ok(async move { future2.await;});
    // pool.spawn_ok(future3); // not sure, why, but also works without the async move
  // 3. using futures block_on to spawn Synchronously
    // use futures::executor::block_on;
    // block_on(async move { future3.await;});
  // 4. using smoll
    // smoll::block_on(async move { future.await;});
  // 5. using wasm_bindgen_futures
    // wasm_bindgen_futures::spawn_local(async move { future.await;});
  // 6. using async_std to run multi-threaded
    // use async_std::task;
    // task::spawn(async move { future.await;});
  // 7. using smol
    // smol::spawn(async move { future.await;});
  // 8. using tokio to run multi-threaded
    // tokio::spawn(async move { future.await;});
  // 9. using wasm_bindgen with main
    // #[wasm_bindgen(start)]
    // pub async main_js() -> Result<(), JsValue> { future.await; Ok(())}
  // 10. using async_std with main
    // #[async_std::main]
    // async fn main() { future.await;}
  // 11. using tokio with main
    // #[tokio::main]
    // async fn main() { future.await;}
}

pub fn mutable_observe() { // example of how to actually run the code so the callbacks get called
  // use futures_signals::signal::Mutable;
  // use futures_signals::signal::SignalExt; //for Iterator trait (gives for_each)
  use futures::executor::LocalPool;
  use std::thread;
  use futures::join;

    //create my_state, and a clone that will be moved to the thread
  let my_state       	= Mutable::new(5);
  let my_state_shared	= my_state.clone();

  thread::spawn(move || { // increment my_state by 1 in a loop, until it reaches 10
    loop {
      my_state_shared.set(my_state_shared.get() + 1);
      thread::sleep(std::time::Duration::from_secs(1));
      if my_state_shared.get() == 10 {
        break;
      }
    } //my_state_shared dropped here
  });

  //create observers
  let obs_a_future = my_state.signal().for_each(|val| { println!("Observer A {}", val); async{} });
  let obs_b_future = my_state.signal().for_each(|val| { println!("Observer B {}", val); async{} });

  drop(my_state); // decrement ref count by one (my_state_shared is still active)

  let mut pool = LocalPool::new(); // run the app until my_state_shared is dropped.
  pool.run_until( async { join!(obs_a_future, obs_b_future); });
  println!("finished!");
  // all references to the mutable had to be dropped before the future it created is marked as complete
  // As long as you have a reference to the Mutable, it's possible to change the value, and so the receiver has to assume that more changes might happen
  // What if you don't want to wait for the Mutable to drop? What if you want to stop listening earlier? In that case you can cancel the Future by using abortable
    // spawner.spawn_local(async move { abort_future.await;});
    // abort_handle.abort(); // stop the Future even if the Mutable still exists
  // use futures::future::abortable;
  // let future2 = my_state.signal().for_each(|v| { println!("@f2 v={}", v); async {} });
  // let (abort_future, abort_handle) = abortable(future2);
}
pub fn mutable_observe2() { // example of how to actually run the code so the callbacks get called
  use std::thread::sleep;
  use std::time;

  // Timeout
    // will run the some_signal.for_each(...) Future and the sleep(2000.0) Future in parallel
    // After 1 second, the sleep Future will finish, and then it will cancel the for_each Future
    // like abortable, this works on any Future, so every Future in Rust supports timeouts. Various crates like async-std have a timeout function which behaves just like the select code
  let my_state	= Mutable::new(11);
  let future_t1 = my_state.signal().for_each(|v| { println!("@future_t1 v={}", v); async {} });
  // let future_t1 = async { futures::future::pending::<()>().await; "ret <>" }; // never finish
  // let future_t2 = async { futures::future::ready(22).await }; // finished right away
  // let future_t2 = async { sleep(time::Duration::from_millis(2000)); "ret after 2000ms" }; // finished after 2sec
  let future_t2 = async { sleep(time::Duration::from_millis(2000)); () }; // make compat with signal
  // let mut future_t1b = Box::pin(future_t1);
  // let mut future_t2b = Box::pin(future_t2);
  futures::pin_mut!(future_t1);
  futures::pin_mut!(future_t2);
  let future_timeout = futures::future::select(future_t1, future_t2);
  use futures::{pin_mut, future::Either, future::self};
  let future = async { let value = match future_timeout.await {
      Either::Left( (val1,_)) => val1, // `val1` resolved from `fut1`; `_` represents `fut2`
      Either::Right((val2,_)) => val2, // `val2` resolved from `fut2`; `_` represents `fut1`
    };
    // println!("   ×××@future_timeout match: {}", &value);
  };
  // use futures::executor::ThreadPool;
  // let pool1 = ThreadPool::new().unwrap();
  // pool1.spawn_ok(async move { future.await;}); // pin_mut!(future_t1) borrowed value does not live long enough; but Box::pin(future_t1); works
  futures::executor::block_on(async move { future.await;});
}

async fn print_async() { println!("Hello from print_async") }
  // async fn foo(args..) -> T is a function of the type fn(args..) -> impl Future<Output = T>
  // return type is an anonymous type generated by the compiler
pub fn async_example() { // An async function is a kind of delayed computation - nothing in the body of the function actually runs until you begin polling the future returned by the function
  let future = print_async();
  println!("Hello from main 1");
  futures::executor::block_on(future);

  // 2. Async closure unstable https://github.com/rust-lang/rust/issues/62290
    // async closures can be annotated with move to capture ownership of the variables they close over
  // let async_closure = async || { println!("Hello from async closure."); };
  // println!("Hello from main 2");
  // let async_closure_future = async_closure();
  // println!("Hello from main 3");
  // futures::executor::block_on(async_closure_future);

  // 3. Async blocks. Almost equivalent to an immediately-invoked async closure
    //  async    { /* body */ }    // is equivalent to
    // (async || { /* body */ })() // except that control-flow constructs (return, break, continue) (unless they appear within a fresh control-flow context like a closure or a loop) are not allowed within the body
    // async blocks can be annotated with move to capture ownership of the variables they close over

  println!("Hello from main 2");
  let async_block_future = async { println!("Hello from an async block"); };
  println!("Hello from main 3");
  // futures::executor::block_on(async_block_future);

  // let n = (async_block_future).await; // only available inside async functions


  use futures::executor::block_on; use futures::future::{self, FutureExt};
  block_on(async {
    let fut = future::lazy(|_| vec![0, 1, 2, 3]);
    let shared1 = fut.shared();
    let shared2 = shared1.clone();
    let shared1_await = shared1.await;
    let shared2_await = shared2.await;
    println!("{:?}", &shared1_await);      	// [0, 1, 2, 3]
    println!("{:?}", &shared1_await.len());	// 4
    println!("{:?}", &shared2_await);      	// [0, 1, 2, 3]
    println!("{:?}", &shared2_await.len());	// 4
  })
}
