use std::net::Shutdown::Read;
use std::rc::Rc;


pub struct ReaderMonad<Tcfg, Ta> {
    pub run_reader: Rc<dyn Fn(Tcfg) -> Ta>
}


pub fn reader_unit<Tcfg, Ta: 'static + Copy>(a: Ta) -> ReaderMonad<Tcfg, Ta> {
    ReaderMonad { run_reader: Rc::new(move |cfg| a) }
}


pub fn reader_fmap<Tcfg: 'static, Ta: 'static, Tb: 'static>(
    f_ab: Rc<dyn Fn(Ta) -> Tb>,
    ma: ReaderMonad<Tcfg, Ta>
) -> ReaderMonad<Tcfg, Tb> {
    ReaderMonad { run_reader: Rc::new( move |cfg| f_ab((ma.run_reader)(cfg)) ) }
}


pub fn reader_apply<Tcfg: 'static + Copy, Ta: 'static, Tb: 'static>(
    mf: ReaderMonad<Tcfg, Rc<dyn Fn(Ta) -> Tb>>,
    ma: ReaderMonad<Tcfg, Ta>
) -> ReaderMonad<Tcfg, Tb> {
    ReaderMonad { run_reader: Rc::new( move |cfg| ((mf.run_reader)(cfg))((ma.run_reader)(cfg)) ) }
}

pub struct ReaderKleisli<Tcfg, Ta, Tb> {
    pub kleisli: Rc<dyn Fn(Ta) -> ReaderMonad<Tcfg, Tb>>
}

pub fn reader_bind<Tcfg: 'static + Copy, Ta: 'static, Tb: 'static>(
    ma: ReaderMonad<Tcfg, Ta>,
    k_ab: ReaderKleisli<Tcfg, Ta, Tb>
) -> ReaderMonad<Tcfg, Tb> {
    ReaderMonad { run_reader: Rc::new( move |cfg| ((k_ab.kleisli)((ma.run_reader)(cfg)).run_reader)(cfg) ) }
}


pub fn load<Tcfg>() -> ReaderMonad<Tcfg, Tcfg> {
    ReaderMonad { run_reader: Rc::new(|cfg| cfg) }
}


#[macro_export]
macro_rules! reader_do {
    ($v:ident = $e:expr,  $($rest:tt)*) => { (|$v| { reader_do!($($rest)*) })($e) };

    ($v:ident <- $e:expr, $($rest:tt)*) => {
        reader_bind(
            $e,
            ReaderKleisli {
                kleisli: Rc::new( move |$v| { reader_do!($($rest)*) } )
            }
        )
    };

    ($e:expr, $($rest:tt)*) => {
        reader_bind(
            $e,
            ReaderKleisli {
                kleisli: Rc::new( move |_| { reader_do!($($rest)*) } )
            }
        )
    };

    ($e:expr) => { $e };
}

// X <- load()
// return X
//
// load() >>= (\X -> return X)

    // ReaderMonad { run_reader: move |cfg| (return (load().run_reader)(cfg)).run_reader)(cfg)  }
