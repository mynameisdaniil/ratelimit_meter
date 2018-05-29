use std::fmt;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
/// Wraps the atomic operations on a Decider's state in a threadsafe
/// fashion.
pub(crate) struct ThreadsafeWrapper<T>
where
    T: fmt::Debug + Default + Clone,
{
    data: Arc<Mutex<T>>,
}

impl<T> Default for ThreadsafeWrapper<T>
where
    T: fmt::Debug + Default + Clone,
{
    fn default() -> Self {
        ThreadsafeWrapper {
            data: Arc::new(Mutex::new(T::default())),
        }
    }
}

impl<T> fmt::Debug for ThreadsafeWrapper<T>
where
    T: fmt::Debug + Default + Clone,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let data = self.data.lock();
        data.fmt(f)
    }
}

impl<T> ThreadsafeWrapper<T>
where
    T: fmt::Debug + Default + Clone,
{
    pub(crate) fn new(elt: T) -> ThreadsafeWrapper<T> {
        ThreadsafeWrapper {
            data: Arc::new(Mutex::new(elt)),
        }
    }

    #[inline]
    /// Wraps retrieving a bucket's data, calls a function to make a
    /// decision and return a new state, and then tries to set the
    /// state on the bucket.
    ///
    /// This function can loop and call the decision closure again if
    /// the bucket state couldn't be set.
    ///
    /// # Panics
    /// Panics if an error occurs in acquiring any locks.
    pub(crate) fn measure_and_replace<F, E>(&mut self, f: F) -> Result<(), E>
    where
        F: Fn(&T) -> (Result<(), E>, Option<T>),
    {
        let mut data = self.data.lock().unwrap();
        let (decision, new_data) = f(&*data);
        if let Some(new_data) = new_data {
            *data = new_data;
        }
        decision
    }

    /// Retrieves and returns a snapshot of the bucket state. This
    /// isn't thread safe, but can be used to restore an old copy of
    /// the bucket if necessary.
    ///
    /// # Thread safety
    /// This function operates threadsafely, but you're literally
    /// taking a copy of data that will change. Relying on the data
    /// that is returned is *not* threadsafe.
    ///
    /// # Panics
    /// Panics if an error occurs in acquiring any locks.
    pub(crate) fn snapshot(&self) -> T {
        let data = self.data.lock().unwrap();
        data.clone()
    }
}