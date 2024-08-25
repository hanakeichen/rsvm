use clap::Parser;
use rsvm::{
    thread::Thread,
    value::JValue,
    vm::{VMConfig, VM},
    JArray,
};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Class search path of directories and jar files
    #[arg(short, long)]
    class_path: Option<String>,

    /// The main class
    main_class: String,
}

fn main() {
    env_logger::init();

    let cli = Cli::parse();
    let mut cfg = VMConfig::default();
    if let Some(cp) = cli.class_path {
        cfg.set_class_path(&cp);
    };
    let mut vm = VM::new(&cfg);

    let thread = std::thread::Builder::new()
        .stack_size(4 * 1024 * 1024)
        .name("main".to_string())
        .spawn(move || {
            vm.init().unwrap();

            let main_class = cli.main_class.as_str();

            let class = vm
                .bootstrap_class_loader
                .load_binary_name_class(main_class)
                .unwrap();

            let method = vm
                .get_static_method(class, "main", "([Ljava/lang/String;)V", Thread::current())
                .unwrap();
            let args = JArray::new_obj_arr(1, Thread::current());
            vm.call_static_void(class, method, &[JValue::with_obj_val(args.cast())]);
        })
        .unwrap();

    thread.join().unwrap();
}
