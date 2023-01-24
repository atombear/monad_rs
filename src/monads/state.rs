use std::rc::Rc;

pub struct StateMonad<Ts, Ta> {
    pub run_state: Rc<dyn Fn(Ts) -> (Ta, Ts) >
}


pub fn state_unit<Ts: 'static + Clone, Ta: 'static + Copy>(a: Ta) -> StateMonad<Ts, Ta> {
    StateMonad { run_state: Rc::new( move |s: Ts| -> (Ta, Ts) { (a, s.clone()) } ) }
}


// functor
pub fn state_fmap<Ts: 'static, Ta: 'static, Tb: 'static>(
    f_ab: Rc<dyn Fn(Ta) -> Tb>,
    ma: StateMonad<Ts, Ta>
) -> StateMonad<Ts, Tb> {
    StateMonad { run_state:
        Rc::new( move |s: Ts| -> (Tb, Ts) {
            let a_s: (Ta, Ts) = (ma.run_state)(s);
            (f_ab(a_s.0), a_s.1)
        })
    }
}


// applicative
pub fn state_apply<Ts: 'static, Ta: 'static, Tb: 'static>(
    mf: StateMonad<Ts, Rc<dyn Fn(Ta) -> Tb>>,
    ma: StateMonad<Ts, Ta>
) -> StateMonad<Ts, Tb> {
    StateMonad { run_state:
        Rc::new( move |s: Ts| -> (Tb, Ts) {
            let f_s: (Rc<dyn Fn(Ta) -> Tb>, Ts) = (mf.run_state)(s);
            let a_s: (Ta, Ts) = (ma.run_state)(f_s.1);
            ((f_s.0)(a_s.0), a_s.1)
        })
    }
}


// monad
pub struct StateKleisli<Ts, Ta, Tb> {
    pub kleisli: Rc<dyn Fn(Ta) -> StateMonad<Ts, Tb>>
}

pub fn state_bind<Ts: 'static, Ta: 'static, Tb: 'static>(
    ma: StateMonad<Ts, Ta>,
    k_ab: StateKleisli<Ts, Ta, Tb>
) -> StateMonad<Ts, Tb> {
    StateMonad { run_state:
        Rc::new( move |s: Ts| -> (Tb, Ts) {
            let a_s: (Ta, Ts) = (ma.run_state)(s);
            let mb: StateMonad<Ts, Tb> = (k_ab.kleisli)(a_s.0);
            (mb.run_state)(a_s.1)
        })
    }
}


// extracts the state from the monadic context.
pub fn get<Ts: Clone>() -> StateMonad<Ts, Ts> {
    StateMonad { run_state: Rc::new( move |s: Ts| -> (Ts, Ts) {
        let s0: Ts = s.clone();
        let s1: Ts = s.clone();
        (s0, s1)
    }) }
}


// returns the state into the monadic context
pub fn put<Ts: 'static + Clone>(s: Ts) -> StateMonad<Ts, ()> {
    StateMonad { run_state: Rc::new( move |_: Ts| -> ((), Ts) { ((), s.clone()) } ) }
}


#[macro_export]
macro_rules! state_do {
    // trailing comma
    () => {};

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


// tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fmap() {
        let s0 = state_fmap(
            Rc::new(|x| x + 1),
            StateMonad { run_state: Rc::new(|s| (10, s)) }
        );
        assert_eq!((s0.run_state)((0, 0)), (11, (0, 0)));
    }

    #[test]
    fn test_apply() {
        let s0 = state_apply(
            StateMonad { run_state: Rc::new(|s| (Rc::new(|x| x + 2), s)) },
            StateMonad { run_state: Rc::new(|s| (10, s)) }
        );
        assert_eq!((s0.run_state)((0, 0)), (12, (0, 0)));
    }

    #[test]
    fn test_bind() {
        let s0 = state_bind(
            StateMonad { run_state: Rc::new(|s| (10, s)) },
            StateKleisli {
                kleisli: Rc::new( move |x|
                    StateMonad { run_state: Rc::new(move |s| (2 * x, s))
                })
            }
        );
        assert_eq!((s0.run_state)((0, 0)), (20, (0, 0)));
    }

    #[test]
    fn test_do() {
        let run_game: StateMonad<(i64, i64), i64> = state_do!(
            st <- get(),
            winner = 0,
            put(if winner == 0 { (st.0+1, st.1) } else { (st.0, st.1+1) }),
            state_unit(winner),
        );
        assert_eq!((run_game.run_state)((10, 13)), (0, (11, 13)));
    }
}
