use std::rc::Rc;

mod writer;
use writer::{WriterKleisli, WriterMonad, writer_unit, writer_bind, compose_writers, log, StringLog};

mod reader;
use reader::{ReaderMonad, reader_fmap, reader_unit, reader_bind, load, ReaderKleisli};

fn main() {

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
}
