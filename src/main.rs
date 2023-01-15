use std::rc::Rc;

mod writer;

use writer::{WriterKleisli, WriterMonad, writer_unit, writer_bind, compose_writers};

fn main() {

    fn add1_function(x: i64) -> WriterMonad<i64> {
        return (x + 1, "added 1".to_string())
    }
    let add1 = WriterKleisli { kleisli: Rc::new(add1_function) };

    let times2 = WriterKleisli { kleisli: Rc::new(|x: i64| -> WriterMonad<i64> { (2 * x, "multiplied by 2".to_string()) })};

    let add1_times2: WriterKleisli<i64, i64> = compose_writers(add1, times2);

    let val_log: WriterMonad<i64> = writer_bind(writer_unit(10), add1_times2);

    println!("{}", val_log.1);

    // let m0 = writer_unit(11);
    let k0 = WriterKleisli { kleisli: Rc::new(
        |x: i64| -> WriterMonad<i64> { (3*x, "x3".to_string()) }
    ) };
    let k1 = k0.clone();
    let result = writer_binds!( { writer_unit(11) } >>= { k0 } >>= { k1 });
    println!("{:?}", result);
}
