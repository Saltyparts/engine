use rustpython_vm as vm;

fn main() {
    let mut args = std::env::args();
    args.next();

    let game_path = args.next().expect("no game path supplied");
    let game = std::fs::read_to_string(game_path).expect("expected game file, got something else");

    vm::Interpreter::default().enter(|vm| {
        let scope = vm.new_scope_with_builtins();

        let code_obj = vm
            .compile(
                &game,
                vm::compile::Mode::Exec,
                "<embedded>".to_owned(),
            )
            .map_err(|err| vm.new_syntax_error(&err)).expect("syntax error");

        vm.run_code_obj(code_obj, scope).expect("runtime error");
    });
}
