#[macro_use]
extern crate log;

#[macro_use]
mod util_macros;

use hastur::{NameCache, Engine};
use std::fs::File;

macro_rules! variable_set_to (
    ($names:expr, $engine:expr, $variable_name:expr, $value:expr) => {{
        let names = &mut $names;
        let engine = &$engine;
        let variable_name = $variable_name;
        let value = $value;

        let variable_name = names
            .variable_name(variable_name)
            .expect(&format!(
                "Variable named {:?} should have been interned",
                variable_name
            ));

        assert_eq!(
            &engine
                .database
                .get_variable(variable_name)
                .expect(&format!("Variable named {:?} should have a value", variable_name))
                .expand(names, &engine.database).1,
            value
        )
    }}
);

#[test]
fn simple_eval() {
    env_logger::builder().is_test(true).init();

    {
        let mut cwd = std::env::current_dir().unwrap();
        cwd.push("tests");
        cwd.push("eval");
        assert!(std::env::set_current_dir(&cwd).is_ok());
    }

    let mut engine = Engine::default();
    let mut names = NameCache::default();

    let inf = File::open("eval0.mk").expect("Failed to open eval0.mk");
    let mut bufreader = std::io::BufReader::new(inf);

    engine.read_makefile(&mut names, &mut bufreader, "eval0.mk");

    variable_set_to!(names, engine, "foo", "");
}
