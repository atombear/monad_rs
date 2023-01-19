use std::rc::Rc;

pub struct StateMonad<Ts, Ta> {
    pub run_state: Rc<dyn Fn(Ts) -> (Ta, Ts) >
}


pub fn state_unit<Ts: 'static + Clone, Ta: 'static + Copy>(a: Ta) -> StateMonad<Ts, Ta> {
    StateMonad { run_state: Rc::new( move |s| (a, s.clone()) ) }
}


pub fn state_fmap<Ts: 'static, Ta: 'static, Tb: 'static>(
    f_ab: Rc<dyn Fn(Ta) -> Tb>,
    ma: StateMonad<Ts, Ta>
) -> StateMonad<Ts, Tb> {
    StateMonad { run_state:
        Rc::new( move |s| {
            let a_s: (Ta, Ts) = (ma.run_state)(s);
            (f_ab(a_s.0), a_s.1)
        })
    }
}

pub fn state_apply<Ts: 'static, Ta: 'static, Tb: 'static>(
    mf: StateMonad<Ts, Rc<dyn Fn(Ta) -> Tb>>,
    ma: StateMonad<Ts, Ta>
) -> StateMonad<Ts, Tb> {
    StateMonad { run_state:
        Rc::new( move |s| {
            let f_s: (Rc<dyn Fn(Ta) -> Tb>, Ts) = (mf.run_state)(s);
            let a_s: (Ta, Ts) = (ma.run_state)(f_s.1);
            ((f_s.0)(a_s.0), a_s.1)
        })
    }
}


pub struct StateKleisli<Ts, Ta, Tb> {
    pub kleisli: Rc<dyn Fn(Ta) -> StateMonad<Ts, Tb>>
}


pub fn state_bind<Ts: 'static, Ta: 'static, Tb: 'static>(
    ma: StateMonad<Ts, Ta>,
    k_ab: StateKleisli<Ts, Ta, Tb>
) -> StateMonad<Ts, Tb> {
    StateMonad { run_state:
        Rc::new( move |s| {
            let a_s: (Ta, Ts) = (ma.run_state)(s);
            let mb: StateMonad<Ts, Tb> = (k_ab.kleisli)(a_s.0);
            (mb.run_state)(a_s.1)
        })
    }
}


pub fn get<Ts: Clone>() -> StateMonad<Ts, Ts> {
    StateMonad { run_state: Rc::new( move |s| {
        let s0 = s.clone();
        let s1 = s.clone();
        (s0, s1)
    }) }
}


pub fn put<Ts: 'static + Clone>(s: Ts) -> StateMonad<Ts, ()> {
    StateMonad { run_state: Rc::new( move |_| ((), s.clone()) ) }
}


#[macro_export]
macro_rules! state_do {
    ($v:ident = $e:expr,  $($rest:tt)*) => { (|$v| { state_do!($($rest)*) })($e) };

    ($v:ident <- $e:expr, $($rest:tt)*) => {
        state_bind(
            $e,
            StateKleisli {
                kleisli: Rc::new( move |$v| { state_do!($($rest)*) } )
            }
        )
    };

    ($e:expr, $($rest:tt)*) => {
        state_bind(
            $e,
            StateKleisli {
                kleisli: Rc::new( move |_| { state_do!($($rest)*) } )
            }
        )
    };

    ($e:expr) => { $e };
}
