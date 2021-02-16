use num_traits::{One, Zero};

pub(crate) trait FromBool {
    fn from_bool(x: bool) -> Self;
}

impl<T> FromBool for T
where
    T: Zero + One,
{
    fn from_bool(x: bool) -> Self {
        if x {
            One::one()
        } else {
            Zero::zero()
        }
    }
}
