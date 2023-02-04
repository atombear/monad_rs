use std::rc::Rc;


#[derive(Clone)]
pub struct ContMonad<Tr, Ta> {
    run_cont: Rc<dyn Fn(Rc<dyn Fn(Ta) -> Tr>) -> Tr>
}


pub fn cont_unit<Tr, Ta: 'static + Clone>(a: Ta) -> ContMonad<Tr, Ta>{
    ContMonad { run_cont: Rc::new(
        move |f: Rc<dyn Fn(Ta) -> Tr>| -> Tr { f(a.clone()) }
    ) }
}


pub fn cont_eval<Tr>(
    ma: ContMonad<Tr, Tr>
) -> Tr {
    (ma.run_cont)(Rc::new( move |x| x))
}


// functor
// a -> b
// (a -> r) -> r
//
// (b -> r) -> r
//
// the new monad accepts k :: (b -> r), which can be used to create
// a lambda g :: (a -> r), \a -> k $ f_ab a
// g is then passed to ma to return r
pub fn cont_fmap<Tr: 'static, Ta: 'static, Tb: 'static>(
    f_ab: Rc<dyn Fn(Ta) -> Tb>,
    ma: ContMonad<Tr, Ta>
) -> ContMonad<Tr, Tb> {
    ContMonad { run_cont: Rc::new( move |f_br: Rc<dyn Fn(Tb) -> Tr>| -> Tr {
        let f_ab_clone: Rc<dyn Fn(Ta) -> Tb> = f_ab.clone();
        let f_ar: Rc<dyn Fn(Ta) -> Tr> = Rc::new(move |a: Ta| -> Tr { f_br(f_ab_clone(a)) } );
        (ma.run_cont)(f_ar)
    } ) }
}


// applicative
// ((a->b) -> r) -> r
// (   a   -> r) -> r
//
// (   b   -> r) -> r
//
// the new monad accepts k :: (b -> r). The given monads require inputs
// ((a->b)->r) and (a->r).
// these are constructed as:
// g = \a -> f_br(f_ab(a))
// f = \f_ab -> ma(g[f_br, f_ab])
// and the resulting monad is
// \f_br -> mf(f[f_br]) :: r
pub fn cont_apply<Tr: 'static, Ta: 'static, Tb: 'static>(
    mf: ContMonad<Tr, Rc<dyn Fn(Ta) -> Tb>>,
    ma: ContMonad<Tr, Ta>
) -> ContMonad<Tr, Tb> {
    let f_abrr: Rc<dyn Fn(Rc<dyn Fn(Rc<dyn Fn(Ta) -> Tb>) -> Tr>) -> Tr> = mf.run_cont.clone();

    ContMonad { run_cont: Rc::new( move |f_br: Rc<dyn Fn(Tb) -> Tr>| -> Tr {
        let f_arr: Rc<dyn Fn(Rc<dyn Fn(Ta) -> Tr>) -> Tr> = ma.run_cont.clone();

        f_abrr(Rc::new(move |f_ab: Rc<dyn Fn(Ta) -> Tb>| -> Tr {
            let f_br_clone: Rc<dyn Fn(Tb) -> Tr> = f_br.clone();

            f_arr(Rc::new( move |a: Ta| -> Tr {
                f_br_clone(f_ab(a))
            } ) )
        } ) )
    } ) }
}


// monad
pub struct ContKleisli<Tr, Ta, Tb> {
    pub kleisli: Rc<dyn Fn(Ta) -> ContMonad<Tr, Tb>>
}

// ((a -> r) -> r)
// a -> ((b -> r) -> r)
//
// ((b -> r) -> r)
// the kleisli k_ab is used to build a function
// (a -> r) that is given as an input to the first monad ma.
// g :: (a -> r) = \a -> (k_ab a)(f_br)
// the resulting monad is created as
// \f_br -> ma(g[k_ab, f_br])
pub fn cont_bind<Tr: 'static, Ta: 'static, Tb: 'static>(
    ma: ContMonad<Tr, Ta>,
    k_ab: ContKleisli<Tr, Ta, Tb>
) -> ContMonad<Tr, Tb> {
    ContMonad { run_cont: Rc::new( move |f_br: Rc<dyn Fn(Tb) -> Tr>| {
        let f_a_mbrr: Rc<dyn Fn(Ta) -> ContMonad<Tr, Tb>> = (k_ab.kleisli).clone();

        let f_ar: Rc<dyn Fn(Ta) -> Tr> = Rc::new( move |a: Ta| -> Tr {
            let f_br_clone: Rc<dyn Fn(Tb) -> Tr> = f_br.clone();
            (f_a_mbrr(a).run_cont)(f_br_clone)
        });
        (ma.run_cont)(f_ar)
    } ) }
}


#[macro_export]
macro_rules! cont_do {
    // trailing comma
    () => {};

    ($v:ident = $e:expr,  $($rest:tt)*) => { (|$v| { cont_do!($($rest)*) })($e) };

    ($v:ident <- $e:expr, $($rest:tt)*) => {
        cont_bind(
            $e,
            ContKleisli {
                kleisli: Rc::new( move |$v| { cont_do!($($rest)*) } )
            }
        )
    };

    ($e:expr, $($rest:tt)*) => {
        cont_bind(
            $e,
            ContKleisli {
                kleisli: Rc::new( move |_| { cont_do!($($rest)*) } )
            }
        )
    };

    ($e:expr) => { $e };
}


// tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monad() {
        let m0 = cont_unit(10);
        assert_eq!(cont_eval(m0), 10);
    }

    #[test]
    fn test_fmap() {
        let m0 = cont_fmap(
            Rc::new(|x| x + 1),
            cont_unit(10)
        );
        assert_eq!(cont_eval(m0), 11);
    }

    #[test]
    fn test_apply() {
        let m0 = cont_apply(
            cont_unit(Rc::new( move |x| if x == 10 { "equal" } else { if x < 10 { "less" } else { "more" } } )),
            cont_unit(10)
        );
        assert_eq!(cont_eval(m0), "equal");

        let m1 = cont_apply(
            cont_unit(Rc::new( move |x| if x == 10 { "equal" } else { if x < 10 { "less" } else { "more" } } )),
            cont_unit(9)
        );
        assert_eq!(cont_eval(m1), "less");

        let m2 = cont_apply(
            cont_unit(Rc::new( move |x| if x == 10 { "equal" } else { if x < 10 { "less" } else { "more" } } )),
            cont_unit(12)
        );
        assert_eq!(cont_eval(m2), "more");

        let m3 = cont_apply(
            cont_unit(Rc::new( move |x| if x == 10 { "equal" } else { if x < 10 { "less" } else { "more" } } )),
            cont_unit(12)
        );
        let m4 = cont_fmap(Rc::new( move |x: &str| x.len()), m3);
        assert_eq!(cont_eval(m4), 4);
    }

    #[test]
    fn test_bind() {
        let m0 = cont_bind(
            cont_unit(100),
            ContKleisli { kleisli: Rc::new( move |x| cont_unit(x * 100))}
        );
        assert_eq!(cont_eval(m0), 10000);
    }

    #[test]
    fn test_do() {
        let calc = cont_do!(
            x <- cont_unit(10),
            y <- cont_unit(4 * x),
            z <- cont_unit(y + 11),
            cont_unit(z)
        );
        assert_eq!(cont_eval(calc), 51);

        let do_calc0 = |c| cont_do!(
            x0 <- c,
            x1 <- cont_unit(10 * x0),
            x2 <- cont_unit(x1 + 12),
            cont_unit(x2)
        );
        let partial: ContMonad<i32, i32> = do_calc0(cont_unit(10));
        assert_eq!(cont_eval(partial), 112);
    }
}