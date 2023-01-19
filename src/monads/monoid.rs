pub trait Monoid {
    type T;
    fn mempty() -> Self::T;
    fn mappend(&self, other: Self::T) -> Self::T;
}
