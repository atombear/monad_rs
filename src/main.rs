use std::rc::Rc;

mod monads;
use crate::monads::writer::{WriterKleisli, WriterMonad, writer_unit, writer_fmap, writer_apply, writer_bind, compose_writers, log, StringLog};
use crate::monads::reader::{ReaderKleisli, ReaderMonad, reader_unit, reader_fmap, reader_apply, reader_bind, load};
use crate::monads::state::{StateKleisli, StateMonad, state_unit, state_fmap, state_apply, state_bind, get, put};


fn main() {
    ///////////////////// Writer //////////////////////////////////

    assert_eq!(
        writer_fmap(|x| 2 * x, (5, "hello".to_string())),
        (10, "hello".to_string()));
    assert_eq!(
        writer_fmap(|x| if x == 5 { "zero" } else { "one" }, (5, "hello".to_string())),
        ("zero", "hello".to_string())
    );
    assert_eq!(
        writer_apply((|x| 2*x, "hello".to_string()), (3, "goodbye".to_string())),
        (6, "hello\ngoodbye".to_string())
    );
    assert_eq!(
        writer_bind(
            (1, "hello".to_string()),
            WriterKleisli { kleisli: Rc::new( |x| (2*x, "goodbye".to_string())) }
        ),
        (2, "hello\ngoodbye".to_string())
    );

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

    ///////////////////// Reader //////////////////////////////////

    let r0: ReaderMonad<&str, i64> = reader_fmap(
        Rc::new(|x: i64| 3*x),
        ReaderMonad { run_reader: Rc::new(|cfg| 11) }
    );
    let r1: ReaderMonad<&str, i64> = ReaderMonad{ run_reader: Rc::new(|cfg| 33) };
    assert_eq!((r0.run_reader)("hi"), (r1.run_reader)("hi"));

    let r0: ReaderMonad<&str, i64> = reader_fmap(
        Rc::new(|x: i64| 3*x),
        ReaderMonad { run_reader: Rc::new(|cfg| if cfg == "one" { 10 } else { 1 }) }
    );
    assert_eq!((r0.run_reader)("one"), 30);
    assert_eq!((r0.run_reader)("none"), 3);

    let r0: ReaderMonad<&str, Rc<dyn Fn(i64) -> i64>> = ReaderMonad {
        run_reader: Rc::new( |cfg| {Rc::new(move |x| if cfg == "one" { x } else { 2 * x })})
    };
    let r1: ReaderMonad<&str, i64> = ReaderMonad { run_reader: Rc::new( |cfg| 10 )};
    let r2 = reader_apply(r0, r1);
    assert_eq!((r2.run_reader)("one"), 10);
    assert_eq!((r2.run_reader)("two"), 20);

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

    ///////////////////// State //////////////////////////////////
    


    fn add1_function(x: i64) -> WriterMonad<i64, StringLog> {
        return (x + 1, StringLog { log: "added 1".to_string() })
    }
    let add1 = WriterKleisli { kleisli: Rc::new(add1_function) };

    let times2 = WriterKleisli { kleisli: Rc::new(|x: i64| -> WriterMonad<i64, StringLog> {
        (2 * x, StringLog { log: "multiplied by 2".to_string() })
    })};

    let add1_times2: WriterKleisli<i64, i64, StringLog> = compose_writers(add1, times2);

    let val_log: WriterMonad<i64, StringLog> = writer_bind(writer_unit(10), add1_times2);

    println!("{}", val_log.1.log);


    let k0: WriterKleisli<i64, i64, StringLog> = WriterKleisli { kleisli: Rc::new(
        |x: i64| -> WriterMonad<i64, StringLog> { (3*x, StringLog { log: "x3".to_string() }) }
    ) };
    let k1 = k0.clone();
    let k2 = k0.clone();
    let k3 = k0.clone();
    let k4: WriterKleisli<i64, f64, StringLog> = WriterKleisli { kleisli: Rc::new(
        |x: i64| -> WriterMonad<f64, StringLog> { ((x as f64) / 10., StringLog { log: "div10".to_string() }) }
    ) };
    let result = writer_binds!( { writer_unit(11) } >>= { k0 } >>= { k1 } >>= { k2 } >>= { k3 } >>= { k4 } );
    println!("{:?}", result);


    let f = |x| writer_do!(
        y = 4*x + 7,
        z = y * 10,
        z
    );
    println!("{:?}", f(3));


    let g = |x, y| writer_do!(
        x0 = 4 * x + 7,
        x1 = y * 10,
        x2 = x0 + x1,
        x2
    );
    println!("{:?}", g(3, 8));


    let square_value = |ma: (i64, StringLog) | writer_do!(
        val <- ma,
        val2 = val * val,
        log(StringLog{ log: "squared the number".to_string() }),
        writer_unit(val2)
    );
    println!("{:?}", square_value(writer_unit(13)));


    let add_values = |x: i64| writer_do!(
        log("adding numbers now!".to_string()),
        val = (1..x).sum::<i64>(),
        log("finished adding numbers!".to_string()),
        writer_unit(val)
    );
    println!("{:?}", add_values(30));


    let result: ReaderMonad<(i64, i64, i64), i64> = reader_do!(
        cfg <- load(),
        reader_unit(4 + cfg.1)
    );
    println!("{:?}", (result.run_reader)((0, 1, 2)));

    fn concat<T: Clone>(vec0: &Vec<T>, vec1: &Vec<T>) -> Vec<T> {
        let mut ret = vec0.to_vec();
        for i in vec1.into_iter() {
            ret.push(i.clone());
        }
        return ret
    }

    let act_on_state = state_do!(
        s <- get(),
        x = s[0],
        new_s = concat(&s, &vec![x+1]),
        put(new_s),
        state_unit(x+10)
    );
    println!("{:?}", (act_on_state.run_state)(vec![0]));

}
