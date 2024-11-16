pub(crate) enum ArrayIter<T> {
    One(std::array::IntoIter<T, 1>),
    Two(std::array::IntoIter<T, 2>),
    Thr(std::array::IntoIter<T, 3>),
}

impl<T> From<[T; 1]> for ArrayIter<T> {
    fn from(value: [T; 1]) -> Self {
        Self::One(value.into_iter())
    }
}

impl<T> From<[T; 2]> for ArrayIter<T> {
    fn from(value: [T; 2]) -> Self {
        Self::Two(value.into_iter())
    }
}

impl<T> From<[T; 3]> for ArrayIter<T> {
    fn from(value: [T; 3]) -> Self {
        Self::Thr(value.into_iter())
    }
}

impl<S> Iterator for ArrayIter<S> {
    type Item = S;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            ArrayIter::One(iter) => iter.next(),
            ArrayIter::Two(iter) => iter.next(),
            ArrayIter::Thr(iter) => iter.next(),
        }
    }
}
