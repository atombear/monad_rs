use std::fmt::Display;
use std::rc::Rc;

use super::monoid::{Monoid};

#[derive(Debug, Clone)]
pub struct StringLog {
    pub log: String
}

impl Monoid for StringLog {
    type T = StringLog;
    fn mempty() -> Self::T { StringLog { log: "".to_string() } }
    fn mappend(&self, other: Self::T) -> Self::T { StringLog { log: format!("{}\n{}", self.log, other.log) } }
}

impl Monoid for String {
    type T = String;
    fn mempty() -> Self::T { "".to_string() }
    fn mappend(&self, other: Self::T) -> Self::T { format!("{}\n{}", self, other) }
}


pub type WriterMonad<Ta, Tlog> = (Ta, Tlog);


pub fn writer_unit<Ta: Display + Copy, Tlog: Monoid<T = Tlog>>(a: Ta) -> WriterMonad<Ta, Tlog> {
    (a, <Tlog as Monoid>::mempty())
}


// functor
pub fn writer_fmap<Ta, Tb, Tlog: Monoid>(f_ab: fn(Ta) -> Tb, ma: WriterMonad<Ta, Tlog>) -> WriterMonad<Tb, Tlog> {
    (f_ab(ma.0), ma.1)
}


// applicative
pub fn writer_apply<Ta, Tb, Tlog: Monoid<T = Tlog>>(
    mf: WriterMonad<fn(Ta) -> Tb, Tlog>,
    ma: WriterMonad<Ta, Tlog>
) -> WriterMonad<Tb, Tlog> {
    ((mf.0)(ma.0), mf.1.mappend(ma.1))
}


// monad
#[derive(Clone)]
pub struct WriterKleisli<Ta, Tb, Tlog: Monoid> {
    pub kleisli: Rc<dyn Fn(Ta) -> WriterMonad<Tb, Tlog>>
}

pub fn writer_bind<Ta, Tb, Tlog: Monoid<T = Tlog>>(
    ma: WriterMonad<Ta, Tlog>,
    k_ab: WriterKleisli<Ta, Tb, Tlog>
) -> WriterMonad<Tb, Tlog> {
    let b_log_ab = (k_ab.kleisli)(ma.0);
    (b_log_ab.0, ma.1.mappend(b_log_ab.1))
}

pub fn log<Tlog: Monoid>(msg: Tlog) -> WriterMonad<(), Tlog> {
    ((), msg)
}

#[macro_export]
macro_rules! writer_binds {
    ($m:block) => { $m };
    ($m:block >>= $k:block) => { writer_bind($m, $k) };
    ($m:block >>= $k:block >>= $($rest:tt)*) => { writer_binds!({ writer_bind($m, $k) } >>= $($rest)* ) };
}

#[macro_export]
macro_rules! writer_do {
    ($v:ident = $e:expr,  $($rest:tt)*) => { (|$v| { writer_do!($($rest)*) })($e) };

    ($v:ident <- $e:expr, $($rest:tt)*) => {
        writer_bind(
            $e,
            WriterKleisli {
                kleisli: Rc::new( move |$v| { writer_do!($($rest)*) } )
            }
        )
    };

    ($e:expr, $($rest:tt)*) => {
        writer_bind(
            $e,
            WriterKleisli {
                kleisli: Rc::new( move |_| { writer_do!($($rest)*) } )
            }
        )
    };

    ($e:expr) => { $e };
}


pub fn compose_writers<Ta: 'static, Tb: 'static, Tc: 'static, Tlog: Monoid<T = Tlog> + 'static>(
    wab: WriterKleisli<Ta, Tb, Tlog>, wbc: WriterKleisli<Tb, Tc, Tlog>
) -> WriterKleisli<Ta, Tc, Tlog> {

    WriterKleisli {
        kleisli: Rc::new(move |a: Ta| -> WriterMonad<Tc, Tlog> {
            let b_log_ab = (wab.kleisli)(a);
            let c_log_bc = (wbc.kleisli)(b_log_ab.0);
            (c_log_bc.0, b_log_ab.1.mappend(c_log_bc.1))
        })
    }

}


// tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fmap() {
        assert_eq!(
            writer_fmap(|x| 2 * x, (5, "hello".to_string())),
            (10, "hello".to_string()));
        assert_eq!(
            writer_fmap(|x| if x == 5 { "zero" } else { "one" }, (5, "hello".to_string())),
            ("zero", "hello".to_string())
        );
    }

    #[test]
    fn test_apply() {
        assert_eq!(
            writer_apply((|x| 2*x, "hello".to_string()), (3, "goodbye".to_string())),
            (6, "hello\ngoodbye".to_string())
        );
    }

    #[test]
    fn test_bind() {
        assert_eq!(
            writer_bind(
                (1, "hello".to_string()),
                WriterKleisli { kleisli: Rc::new( |x| (2*x, "goodbye".to_string())) }
            ),
            (2, "hello\ngoodbye".to_string())
        );
    }

    #[test]
    fn test_do() {
        let do_calculation = |x| writer_do!(
            log(format!("received number {}", x)),
            x0 = x + 10,
            log("added 10 to the number".to_string()),
            x1 = 2 * x0,
            log("multiplied result by 2".to_string()),
            writer_unit(x1)
        );
        assert_eq!(
            do_calculation(5),
            (30, "received number 5\nadded 10 to the number\nmultiplied result by 2\n".to_string())
        );
    }
}
