use std::rc::Rc;


pub struct ContMonad<Tr, Ta> {
    run_cont: Rc<dyn Fn(Rc<dyn Fn(Ta) -> Tr>) -> Tr>
}


pub fn cont_unit<Tr, Ta: 'static + Clone>(a: Ta) -> ContMonad<Tr, Ta>{
    ContMonad { run_cont: Rc::new(
        move |f: Rc<dyn Fn(Ta) -> Tr>| -> Tr { f(a.clone()) }
    ) }
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
// (    a  -> r) -> r
//
// (    b  -> r) -> r
//
// the new monad accepts k :: (b -> r). The given monads require inputs
// ((a->b)->r) and (a->r).
// these are constructed as:
// g = \a -> f_br(f_ab(a))
// f = \f_ab -> ma(g[f_br, f_ab])
// and fully constructed monad is
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
