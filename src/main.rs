use deno_lib_ext::deno_runtime::deno_core::error::JsStackFrame;
use deno_lib_ext::deno_runtime::deno_permissions::set_prompter;
use deno_lib_ext::deno_runtime::deno_permissions::PermissionPrompter;
use deno_lib_ext::deno_runtime::deno_permissions::PromptResponse;

deno_core::extension!(
    runtime_extension,
    ops = [op_document_dir],
    esm_entry_point = "ext:runtime_extension/mod.js",
    esm = [dir "src", "mod.js"],
);

#[deno_core::op2]
#[string]
fn op_document_dir() -> String {
    "dirs::document_dir().map(|path| path.to_string_lossy().to_string())".to_string()
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
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

    let result =
        deno_lib_ext::run_file("./test.ts", vec![runtime_extension::init_ops_and_esm()]).await;
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
