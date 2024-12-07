use deno_lib_ext::deno_runtime::deno_core::error::JsStackFrame;
use deno_lib_ext::deno_runtime::deno_permissions::set_prompter;
use deno_lib_ext::deno_runtime::deno_permissions::PermissionPrompter;
use deno_lib_ext::deno_runtime::deno_permissions::PromptResponse;

deno_core::extension!(
    runtime_extension,
    ops = [custom_op_document_dir],
    esm_entry_point = "ext:runtime_extension/mod.js",
    esm = [dir "src", "mod.js"],
);

#[deno_core::op2]
#[string]
fn custom_op_document_dir() -> Option<String> {
    dirs::document_dir().map(|path| path.to_string_lossy().to_string())
}

deno_core::extension!(
    my_ext2,
    ops = [custom_op_my_op2],
    esm_entry_point = "ext:my_ext2/my_ext.js",
    esm = [dir "src", "my_ext.js"],
);

#[deno_core::op2]
#[string]
fn custom_op_my_op2() -> String {
    "my_op2".to_string()
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    // get filename from args
    let args = std::env::args().collect::<Vec<String>>();
    let empty = String::new();
    let filename = args.get(1).unwrap_or(&empty);

    if filename.is_empty() {
        println!("No filename provided");
        return;
    }

    set_prompter(Box::new(CustomPrompter));

    std::env::set_var(
        "DENO_DIR",
        std::env::current_dir()
            .unwrap()
            .join("deno_dir")
            .to_str()
            .unwrap(),
    );

    std::env::set_var("DENO_TRACE_PERMISSIONS", "1");

    println!("Starting...");

    let start = std::time::Instant::now();

    let result = deno_lib_ext::run_file(
        &filename,
        vec![
            runtime_extension::init_ops_and_esm(),
            my_ext2::init_ops_and_esm(),
        ],
    )
    .await;

    let duration = start.elapsed();

    println!("Time elapsed: {:?}", duration);

    println!("Result: {:?}", result);
}

struct CustomPrompter;

impl PermissionPrompter for CustomPrompter {
    fn prompt(
        &mut self,
        message: &str,
        name: &str,
        api_name: Option<&str>,
        is_unary: bool,
        stack: std::option::Option<Vec<JsStackFrame>>,
    ) -> PromptResponse {
        println!(
            "{}\n{} {}\n{} {}\n{} {:?}\n{} {}\n{} {}",
            "Script is trying to access APIs and needs permission:",
            "Message:",
            message,
            "Name:",
            name,
            "API:",
            api_name,
            "Is unary:",
            is_unary,
            "Stack:",
            stack
                .unwrap()
                .iter()
                .map(|frame| {
                    format!(
                        "{}:{}:{}",
                        frame.file_name.clone().unwrap_or("unknown".to_string()),
                        frame.function_name.clone().unwrap_or("unknown".to_string()),
                        frame.line_number.unwrap_or(0)
                    )
                })
                .collect::<Vec<String>>()
                .join("\n"),
        );
        println!("Allow? [y/n]");

        let mut input = String::new();
        if std::io::stdin().read_line(&mut input).is_ok() {
            match input.trim().to_lowercase().as_str() {
                "y" | "yes" => PromptResponse::Allow,
                _ => PromptResponse::Deny,
            }
        } else {
            println!("Failed to read input, denying permission");
            PromptResponse::Deny
        }
    }
}
