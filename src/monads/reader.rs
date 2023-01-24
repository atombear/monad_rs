use std::rc::Rc;


pub struct ReaderMonad<Tcfg, Ta> {
    pub run_reader: Rc<dyn Fn(Tcfg) -> Ta>
}


pub fn reader_unit<Tcfg, Ta: 'static + Clone>(a: Ta) -> ReaderMonad<Tcfg, Ta> {
    ReaderMonad { run_reader: Rc::new(move |cfg: Tcfg| -> Ta { a.clone() } ) }
}


// functor
pub fn reader_fmap<Tcfg: 'static, Ta: 'static, Tb: 'static>(
    f_ab: Rc<dyn Fn(Ta) -> Tb>,
    ma: ReaderMonad<Tcfg, Ta>
) -> ReaderMonad<Tcfg, Tb> {
    ReaderMonad { run_reader: Rc::new( move |cfg: Tcfg| -> Tb { f_ab((ma.run_reader)(cfg)) } ) }
}


// applicative
pub fn reader_apply<Tcfg: 'static + Copy, Ta: 'static, Tb: 'static>(
    mf: ReaderMonad<Tcfg, Rc<dyn Fn(Ta) -> Tb>>,
    ma: ReaderMonad<Tcfg, Ta>
) -> ReaderMonad<Tcfg, Tb> {
    ReaderMonad { run_reader: Rc::new( move |cfg: Tcfg| -> Tb { ((mf.run_reader)(cfg))((ma.run_reader)(cfg)) } ) }
}


// monad
pub struct ReaderKleisli<Tcfg, Ta, Tb> {
    pub kleisli: Rc<dyn Fn(Ta) -> ReaderMonad<Tcfg, Tb>>
}

pub fn reader_bind<Tcfg: 'static + Clone, Ta: 'static, Tb: 'static>(
    ma: ReaderMonad<Tcfg, Ta>,
    k_ab: ReaderKleisli<Tcfg, Ta, Tb>
) -> ReaderMonad<Tcfg, Tb> {
    ReaderMonad { run_reader: Rc::new( move |cfg: Tcfg| -> Tb { ((k_ab.kleisli)((ma.run_reader)(cfg.clone())).run_reader)(cfg.clone()) } ) }
}


// extracts the configuration from the monadic context to be used.
pub fn load<Tcfg>() -> ReaderMonad<Tcfg, Tcfg> {
    ReaderMonad { run_reader: Rc::new(|cfg: Tcfg| -> Tcfg { cfg } ) }
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


// tests
#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_fmap0() {
        let r0: ReaderMonad<&str, i64> = reader_fmap(
            Rc::new(|x: i64| 3*x),
            ReaderMonad { run_reader: Rc::new(|cfg| 11) }
        );
        let r1: ReaderMonad<&str, i64> = ReaderMonad{ run_reader: Rc::new(|cfg| 33) };
        assert_eq!((r0.run_reader)("hi"), (r1.run_reader)("hi"));
    }

    #[test]
    fn test_fmap1() {
        let r0: ReaderMonad<&str, i64> = reader_fmap(
            Rc::new(|x: i64| 3*x),
            ReaderMonad { run_reader: Rc::new(|cfg| if cfg == "one" { 10 } else { 1 }) }
        );
        assert_eq!((r0.run_reader)("one"), 30);
        assert_eq!((r0.run_reader)("none"), 3);
    }

    #[test]
    fn test_apply() {
        let r0: ReaderMonad<&str, Rc<dyn Fn(i64) -> i64>> = ReaderMonad {
            run_reader: Rc::new( |cfg| {Rc::new(move |x| if cfg == "one" { x } else { 2 * x })})
        };
        let r1: ReaderMonad<&str, i64> = ReaderMonad { run_reader: Rc::new( |cfg| 10 )};
        let r2 = reader_apply(r0, r1);
        assert_eq!((r2.run_reader)("one"), 10);
        assert_eq!((r2.run_reader)("two"), 20);
    }

    #[test]
    fn test_bind() {
        let r0: ReaderMonad<(bool, bool), &str> = ReaderMonad { run_reader: Rc::new(|cfg| if cfg.0 { "t" } else { "f" })};
        let k0: ReaderKleisli<(bool, bool), &str, (&str, &str)> = ReaderKleisli {
            kleisli: Rc::new( |l|
                ReaderMonad { run_reader: Rc::new( |cfg| if cfg.1 { (l, "t") } else { (l, "f") } ) }
            )};
        let r1 = reader_bind(r0, k0);
        assert_eq!((r1.run_reader)((true, true)), ("t", "t"));
        assert_eq!((r1.run_reader)((true, false)), ("t", "f"));
        assert_eq!((r1.run_reader)((false, true)), ("f", "t"));
        assert_eq!((r1.run_reader)((false, false)), ("f", "f"));
    }

    #[test]
    fn test_do() {
        #[derive(Debug, Clone)]
        struct User {
            uname: String,
            host: String,
            ip: String,
        }
        let gen_login_str: ReaderMonad<User, String> = reader_do!(
            cfg <- load(),
            reader_unit(format!("{}-{}-{}", cfg.uname, cfg.host, cfg.ip))
        );
        assert_eq!(
            (gen_login_str.run_reader)(User { uname: "a".to_string(), host: "b".to_string(), ip: "c".to_string() }),
            "a-b-c".to_string()
        );
    }
}
