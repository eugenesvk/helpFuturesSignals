//! A very simplified working example of a push+pull system from github.com/Pauan/rust-signals/issues/1#issuecomment-433827027
//! Doesn't handle cancellation, but it does solve the diamond problem (by using a global change ID)
use std::sync::atomic::	{AtomicUsize, Ordering};
use std::sync::        	{Arc, Mutex, RwLock};

pub static CHANGE_ID: AtomicUsize = AtomicUsize::new(0);

pub trait Signal {
  type Item;
  fn with_value<A, F> (&self, f:F) -> A where F:FnOnce(&Self::Item) -> A;
  fn register_listener(&self, f:Arc<dyn Fn(usize)>);
}


pub fn for_each<S, F>(signal:S, f:F)
  where S:Signal       + 'static,
        F:Fn(&S::Item) + 'static {
  let signal = Arc::new(signal);

  signal.with_value(|value| f(value));

  let old_id = Mutex::new(None);

  signal.register_listener({
    let signal = signal.clone();
    Arc::new(move |new_id| {
      let mut old_id = old_id.lock().unwrap();

      let changed = match *old_id {
        None        	=> true,
        Some(old_id)	=> old_id != new_id,
      };

      if changed { // This fixes the diamond problem
        *old_id = Some(new_id);
        signal.with_value(|value| f(value));
      }
    })
  });
}


struct Inner<A> {
  value    	: RwLock<A>,
  listeners	: Mutex<Vec<Arc<dyn Fn(usize)>>>,
}

impl<A> Inner<A> {
  #[inline]
  fn new(value: A) -> Self {
    Self {
      value    	: RwLock::new(value),
      listeners	: Mutex::new(vec![]),
    }
  }
}


#[derive(Clone)]
pub struct Mutable<A> { inner: Arc<Inner<A>>, }

impl<A> Mutable<A> {
  #[inline]
  pub fn new(value: A) -> Self {
    Self { inner:Arc::new(Inner::new(value)), }
  }

  pub fn emit(&self, value: A) {
    let id = CHANGE_ID.fetch_add(1, Ordering::SeqCst); // This fixes the diamond problem

    *self.inner.value.write().unwrap() = value;

    for listener in self.inner.listeners.lock().unwrap().as_mut_slice() {
      listener(id); // It doesn't pass the new value
    }
  }
}

impl<A> Signal for Mutable<A> {
  type Item = A;

  #[inline]
  fn with_value<B, F>(&self, f:F) -> B where F:FnOnce(&Self::Item) -> B {
    let value = self.inner.value.read().unwrap();
    f(&*value)
  }

  fn register_listener(&self, f:Arc<dyn Fn(usize)>) {
    self.inner.listeners.lock().unwrap().push(f);
  }
}


pub struct Map<A, F> {
  signal  	: A,
  callback	: F,
}

impl<A, B, F> Map<A, F>
  where A:Signal,
        F:Fn(&A::Item) -> B {
  #[inline]
  pub fn new(signal:A, callback:F) -> Self {
    Self { signal, callback }
  }
}

impl<A, B, F> Signal for Map<A, F>
  where A:Signal,
        F:Fn(&A::Item) -> B {
  type Item = B;

  #[inline]
  fn with_value<O, G>(&self, f:G) -> O where G:FnOnce(&Self::Item) -> O {
    self.signal.with_value(|value| {
      f(&(self.callback)(value))
    })
  }

  #[inline]
  fn register_listener(&self, f:Arc<dyn Fn(usize)>) {
    self.signal.register_listener(f);
  }
}


pub struct Map2<A, B, F> {
  signal1 	: A,
  signal2 	: B,
  callback	: F,
}

impl<A, B, C, F> Map2<A, B, F>
  where A:Signal,
        B:Signal,
        F:Fn(&A::Item, &B::Item) -> C {
  #[inline]
  pub fn new(signal1:A, signal2:B, callback:F) -> Self {
    Self { signal1, signal2, callback }
  }
}

impl<A, B, C, F> Signal for Map2<A, B, F>
  where A:Signal,
        B:Signal,
        F:Fn(&A::Item, &B::Item) -> C {
  type Item = C;

  #[inline]
  fn with_value<O, G>(&self, f:G) -> O where G:FnOnce(&Self::Item) -> O {
    self.signal1.with_value(|value1| {
      self.signal2.with_value(|value2| {
        f(&(self.callback)(value1, value2))
      })
    })
  }

  #[inline]
  fn register_listener(&self, f:Arc<dyn Fn(usize)>) {
    self.signal1.register_listener(f.clone());
    self.signal2.register_listener(f);
  }
}


fn diamond() {
  let root = Mutable::new(6);

  let left 	= Map:: new(root.clone(), |x|  	{ println!("left changed {}"   , x)   ; *x + 10});
  let right	= Map:: new(root.clone(), |x|  	{ println!("right changed {}"  , x)   ; *x +  2});
  let join 	= Map2::new(left, right , |x,y|	{ println!("join changed {} {}", x, y); *x + *y});
  for_each(join, |value| {println!("{}", value);});

  root.emit(7);
}
