use deno_lib::deno_runtime::deno_permissions::set_prompter;
use deno_lib::deno_runtime::deno_permissions::PermissionPrompter;
use deno_lib::deno_runtime::deno_permissions::PromptResponse;
use deno_lib::run;

fn main() {
    println!("Hello, world!");

    set_prompter(Box::new(CustomPrompter));

    std::env::set_var(
        "DENO_DIR",
        std::env::current_dir()
            .unwrap()
            .join("deno_dir")
            .to_str()
            .unwrap(),
    );

    run("./test.ts");
}

struct CustomPrompter;

impl PermissionPrompter for CustomPrompter {
    fn prompt(
        &mut self,
        message: &str,
        name: &str,
        api_name: Option<&str>,
        is_unary: bool,
    ) -> PromptResponse {
        println!(
            "{}\n{} {}\n{} {}\n{} {:?}\n{} {}",
            "Script is trying to access APIs and needs permission:",
            "Message:",
            message,
            "Name:",
            name,
            "API:",
            api_name,
            "Is unary:",
            is_unary
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
