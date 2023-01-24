// A monoid is defined by a type T of which it
// returns an `empty` version thereof
// can concatenate onto itself.
pub trait Monoid {
    type T;
    fn mempty() -> Self::T;
    fn mappend(&self, other: Self::T) -> Self::T;
}


// containers, in general, are monoids

// strings
impl Monoid for String {
    type T = String;
    fn mempty() -> Self::T { "".to_string() }
    fn mappend(&self, other: Self::T) -> Self::T { format!("{}\n{}", self, other) }
}

// lists
impl<A: Clone> Monoid for Vec<A> {
    type T = Vec<A>;
    fn mempty() -> Self::T { vec![] }
    fn mappend(&self, other: Self::T) -> Self::T {
        let mut ret: Self::T = vec![];
        for el in self { ret.push(el.clone()); }
        for el in other { ret.push(el.clone()); }
        return ret
    }
}
