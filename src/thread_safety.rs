use evmap::ShallowCopy;
use parking_lot::Mutex;
use std::fmt;
use std::sync::Arc;

#[derive(Clone)]
/// Wraps the atomic operations on a Decider's state in a threadsafe
/// fashion.
pub(crate) struct ThreadsafeWrapper<T>
where
    T: fmt::Debug + Default + Clone + PartialEq + Eq,
{
    data: Arc<Mutex<T>>,
}

impl<T> Default for ThreadsafeWrapper<T>
where
    T: fmt::Debug + Default + Clone + PartialEq + Eq,
{
    fn default() -> Self {
        ThreadsafeWrapper {
            data: Arc::new(Mutex::new(T::default())),
        }
    }
}

impl<T> PartialEq<Self> for ThreadsafeWrapper<T>
where
    T: fmt::Debug + Default + Clone + PartialEq + Eq,
{
    fn eq(&self, other: &Self) -> bool {
        let mine = self.data.lock();
        let other = other.data.lock();
        *other == *mine
    }

    fn ne(&self, other: &Self) -> bool {
        let mine = self.data.lock();
        let other = other.data.lock();
        *other != *mine
    }
}

impl<T> Eq for ThreadsafeWrapper<T> where T: fmt::Debug + Default + Clone + PartialEq + Eq {}

impl<T> fmt::Debug for ThreadsafeWrapper<T>
where
    T: fmt::Debug + Default + Clone + PartialEq + Eq,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let data = self.data.lock();
        data.fmt(f)
    }
}

impl<T> ShallowCopy for ThreadsafeWrapper<T>
where
    T: fmt::Debug + Default + Clone + PartialEq + Eq,
{
    unsafe fn shallow_copy(&mut self) -> Self {
        ThreadsafeWrapper {
            data: self.data.shallow_copy(),
        }
    }
}

impl<T> ThreadsafeWrapper<T>
where
    T: fmt::Debug + Default + Clone + PartialEq + Eq,
{
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
        let mut data = self.data.lock();
        let (decision, new_data) = f(&*data);
        if let Some(new_data) = new_data {
            *data = new_data;
        }
        decision
    }
}
