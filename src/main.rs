use clap::{Parser, Subcommand};
use inquire::Select;
use laadlelang::{
    compiler::Compiler, parser::Parser as LaadleParser, tokenizer::Tokenizer,
    vm::LaadleVirtualMachineV1,
};
use rust_embed::RustEmbed;
use rustyline::{DefaultEditor, error::ReadlineError};
use std::fs;
use std::path::PathBuf;
use std::thread;
use tiny_http::{Header, Response, Server};

#[derive(RustEmbed)]
#[folder = "docs/public/playground/"]
struct Assets;

#[derive(Parser)]
#[command(name = "laadlelang")]
#[command(about = "The LaadleLang CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a LaadleLang source file
    Run {
        /// The path to the .laadle file
        path: PathBuf,
    },
    /// Start an interactive REPL session
    Repl,
    /// Launch the LaadleLang Playground in the browser
    Playground,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Run { path }) => {
            run_file(path);
        }
        Some(Commands::Repl) => {
            run_repl();
        }
        Some(Commands::Playground) => {
            launch_playground();
        }
        None => {
            // Interactive menu fallback
            let options = vec!["Start REPL", "Run File", "Launch Playground", "Exit"];
            let ans =
                Select::new("Welcome to LaadleLang! What would you like to do?", options).prompt();

            match ans {
                Ok("Start REPL") => run_repl(),
                Ok("Run File") => {
                    let path_str =
                        inquire::Text::new("Enter the path to your .laadle file:").prompt();
                    match path_str {
                        Ok(p) => run_file(PathBuf::from(p)),
                        Err(_) => {}
                    }
                }
                Ok("Launch Playground") => launch_playground(),
                Ok("Exit") | Err(_) => {
                    println!("Goodbye!");
                }
                _ => {}
            }
        }
    }
}

fn launch_playground() {
    // 1. Find an available port (Default to 3000)
    let port = 3000;
    let addr = format!("127.0.0.1:{}", port);
    let server = match Server::http(&addr) {
        Ok(s) => s,
        Err(_) => {
            // If 3000 is taken, let's try a random port (0 will let OS pick)
            Server::http("127.0.0.1:0").expect("❌ Failed to bind to any port")
        }
    };

    let actual_port = server
        .server_addr()
        .to_ip()
        .map(|i| i.port())
        .unwrap_or(port);
    let url = format!("http://localhost:{}", actual_port);

    println!("🚀 Starting Embedded Playground Server on {}...", url);

    // 2. Spawn a background thread to handle requests
    thread::spawn(move || {
        for request in server.incoming_requests() {
            let url_path = request.url().split('?').next().unwrap_or("/");
            let mut path = url_path.trim_start_matches('/').to_string();

            if path.is_empty() {
                path = "index.html".to_string();
            }

            // Next.js static resolution:
            let (asset, actual_path) = if let Some(a) = Assets::get(&path) {
                (Some(a), path.clone())
            } else if let Some(a) = Assets::get(&format!("{}.html", path)) {
                (Some(a), format!("{}.html", path))
            } else if let Some(a) = Assets::get(&format!("{}/index.html", path)) {
                (Some(a), format!("{}/index.html", path))
            } else {
                (None, path)
            };

            if let Some(data) = asset {
                let mime = mime_guess::from_path(&actual_path).first_or_octet_stream();
                let response = Response::from_data(data.data).with_header(
                    Header::from_bytes(&b"Content-Type"[..], mime.as_ref().as_bytes()).unwrap(),
                );
                let _ = request.respond(response);
            } else {
                let _ = request.respond(Response::from_string("Not Found").with_status_code(404));
            }
        }
    });

    // 3. Launch browser
    std::thread::sleep(std::time::Duration::from_millis(500));
    println!("🌍 Launching Playground IDE at {} ...", url);
    if let Err(e) = open::that(&url) {
        eprintln!("❌ Failed to open browser: {}", e);
        println!("Please open {} manually.", url);
    }

    println!("\n🟢 Embedded Playground is active!");
    println!("🛑 Press Ctrl+C to stop the server and exit.");

    // Block the main thread to keep the server alive
    loop {
        thread::sleep(std::time::Duration::from_secs(60));
    }
}

fn run_file(path: PathBuf) {
    let source = match fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error reading file {:?}: {}", path, e);
            return;
        }
    };

    let mut lexer = Tokenizer::new(&source);
    let tokens = lexer.tokenize();

    let mut parser = LaadleParser::new(tokens);
    let stmts = parser.parse();
    if let Some(err) = parser.error {
        eprintln!("Syntax Error: {}", err);
        return;
    }

    let mut compiler = Compiler::new();
    let program = compiler.compile(&stmts);
    if let Some(err) = compiler.error {
        eprintln!("Compilation Error: {}", err);
        return;
    }

    let mut vm = LaadleVirtualMachineV1::new(program);
    vm.run();
    if let Some(err) = vm.error {
        eprintln!("\n[VM Crash] Uncaught Error: {}", err);
    }
}

fn run_repl() {
    println!("LaadleLang Interactive REPL (Type 'nikal' or Ctrl-C to exit)");
    let mut rl = DefaultEditor::new().expect("Failed to initialize REPL");

    // Persistent VM for the REPL session
    let mut vm = LaadleVirtualMachineV1::new(Vec::new());

    loop {
        let readline = rl.readline("laadle> ");
        match readline {
            Ok(line) => {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }
                if trimmed == "nikal" {
                    break;
                }

                rl.add_history_entry(trimmed).ok();

                let mut lexer = Tokenizer::new(trimmed);
                let tokens = lexer.tokenize();
                let mut parser = LaadleParser::new(tokens);
                let stmts = parser.parse();

                if let Some(err) = parser.error {
                    println!("Syntax Error: {}", err);
                    continue;
                }

                let mut compiler = Compiler::new();
                let mut new_code = compiler.compile(&stmts);
                if let Some(err) = compiler.error {
                    println!("Compilation Error: {}", err);
                    continue;
                }

                // Append and execute
                let start_idx = vm.program.len();
                vm.program.append(&mut new_code);
                vm.ip = start_idx;

                vm.run();

                if let Some(err) = vm.error {
                    println!("[REPL Error] {}", err);
                    vm.error = None;
                }
            }
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => {
                println!("Goodbye!");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
}
