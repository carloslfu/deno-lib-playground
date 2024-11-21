// use deno_lib::deno_runtime::deno_permissions::set_prompter;
// use deno_lib::deno_runtime::deno_permissions::PermissionPrompter;
// use deno_lib::deno_runtime::deno_permissions::PromptResponse;
use deno_lib::run;

fn main() {
    println!("Hello, world!");

    // set_prompter(Box::new(CustomPrompter));

    // set DENO_DIR to current work dir
    std::env::set_var(
        "DENO_DIR",
        std::env::current_dir().unwrap().to_str().unwrap(),
    );

    run("./test.ts");
}

// struct CustomPrompter;

// impl PermissionPrompter for CustomPrompter {
//     fn prompt(
//         &mut self,
//         message: &str,
//         name: &str,
//         api_name: Option<&str>,
//         is_unary: bool,
//     ) -> PromptResponse {
//         println!(
//             "{}\n{} {}\n{} {}\n{} {:?}\n{} {}",
//             "Script is trying to access APIs and needs permission:"
//                 .yellow()
//                 .bold(),
//             "Message:".bright_blue(),
//             message,
//             "Name:".bright_blue(),
//             name,
//             "API:".bright_blue(),
//             api_name,
//             "Is unary:".bright_blue(),
//             is_unary
//         );
//         println!("Allow? [y/n]");

//         let mut input = String::new();
//         if std::io::stdin().read_line(&mut input).is_ok() {
//             match input.trim().to_lowercase().as_str() {
//                 "y" | "yes" => PromptResponse::Allow,
//                 _ => PromptResponse::Deny,
//             }
//         } else {
//             println!("Failed to read input, denying permission");
//             PromptResponse::Deny
//         }
//     }
// }
