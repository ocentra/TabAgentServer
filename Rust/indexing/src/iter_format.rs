use core::fmt;

pub struct NoPretty<T>(pub T);

impl<T> fmt::Debug for NoPretty<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

