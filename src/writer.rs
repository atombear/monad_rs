use std::fmt::Display;
use std::rc::Rc;

pub type WriterMonad<Ta> = (Ta, String);


pub fn writer_unit<Ta: Display + Copy>(a: Ta) -> WriterMonad<Ta> {
    (a, format!("value is {}", a))
}


// functor
fn writer_fmap<Ta, Tb>(f_ab: fn(Ta) -> Tb, ma: WriterMonad<Ta>) -> WriterMonad<Tb> {
    (f_ab(ma.0), ma.1)
}


// applicative
fn writer_applicative<Ta, Tb>(
    mf: WriterMonad<fn(Ta) -> Tb>,
    ma: WriterMonad<Ta>
) -> WriterMonad<Tb> {
    ((mf.0)(ma.0), format!("{}\n{}", mf.1, ma.1))
}


// monad
#[derive(Clone)]
pub struct WriterKleisli<Ta, Tb> {
    pub kleisli: Rc<dyn Fn(Ta) -> WriterMonad<Tb>>
}

pub fn writer_bind<Ta, Tb>(ma: WriterMonad<Ta>, k_ab: WriterKleisli<Ta, Tb>) -> WriterMonad<Tb> {
    let b_log_ab = (k_ab.kleisli)(ma.0);
    (b_log_ab.0, format!("{}\n{}", ma.1, b_log_ab.1))
}

#[macro_export]
macro_rules! writer_binds {
    ($m:block) => { $m };
    ($m:block >>= $k:block) => { writer_bind($m, $k) };
    ($m:block >>= $k:block >>= $rest:tt) => { writer_binds!({ writer_bind($m, $k) } >>= $rest) };
}


pub fn compose_writers<Ta: 'static, Tb: 'static, Tc: 'static>(
    wab: WriterKleisli<Ta, Tb>, wbc: WriterKleisli<Tb, Tc>
) -> WriterKleisli<Ta, Tc> {

    WriterKleisli {
        kleisli: Rc::new(move |a: Ta| -> WriterMonad<Tc> {
            let b_log_ab = (wab.kleisli)(a);
            let c_log_bc = (wbc.kleisli)(b_log_ab.0);
            (c_log_bc.0, format!("{}\n{}", b_log_ab.1, c_log_bc.1))
        })
    }

}
