mod character_tests;
mod helpers;
mod movement_tests;
mod terrain_gen_tests;
mod test_list;

use libtest_mimic::{Arguments, Trial};
use test_list::all_tests;

fn main() {
    // Parse command line arguments
    let args = Arguments::from_args();

    let thread_name =
        std::thread::current().name().map(str::to_string).unwrap();
    assert_eq!(thread_name, "main");

    // Create a list of tests and/or benchmarks (in this case: two dummy tests).
    let all_tests = all_tests();

    {
        let tests = all_tests
            .into_iter()
            .map(|(test_name, test_fn)| {
                Trial::test(test_name, move || {
                    test_fn();
                    Ok(())
                })
            })
            .collect::<Vec<_>>();

        // Run all tests and exit the application appropriatly.
        libtest_mimic::run(&args, tests).exit();
    }
}
