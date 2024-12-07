use std::sync::Arc;

use deno_lib_ext::deno_runtime::deno_core::error::JsStackFrame;
use deno_lib_ext::deno_runtime::deno_permissions::set_prompter;
use deno_lib_ext::deno_runtime::deno_permissions::PermissionPrompter;
use deno_lib_ext::deno_runtime::deno_permissions::PromptResponse;
use deno_lib_ext::deno_runtime::fmt_errors::format_js_error;
use deno_lib_ext::deno_runtime::WorkerExecutionMode;
use deno_lib_ext::factory::CliFactory;
use deno_lib_ext::flags_from_vec;
use deno_lib_ext::get_v8_flags_from_env;
use deno_lib_ext::init_v8_flags;
use deno_lib_ext::tools::run::check_permission_before_script;
use deno_lib_ext::tools::run::maybe_npm_install;
use deno_runtime::deno_core::error::AnyError;
use deno_runtime::deno_core::error::JsError;

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

    let result = run_file(
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

async fn run_file(
    file_path: &str,
    extensions: Vec<deno_runtime::deno_core::Extension>,
) -> Result<i32, AnyError> {
    let args: Vec<_> = vec!["deno", "run", file_path]
        .into_iter()
        .map(std::ffi::OsString::from)
        .collect();

    let flags = resolve_flags_and_init(args)?;

    check_permission_before_script(&flags);

    let factory = CliFactory::from_flags(Arc::new(flags));
    let cli_options = factory.cli_options()?;

    let main_module = cli_options.resolve_main_module()?;

    if main_module.scheme() == "npm" {
        set_npm_user_agent();
    }

    maybe_npm_install(&factory).await?;

    let worker_factory = factory.create_cli_main_worker_factory(Some(false)).await?;

    let mut worker = worker_factory
        .create_main_worker(WorkerExecutionMode::None, main_module.clone(), extensions)
        .await?;

    let exit_code = worker.run().await?;

    Ok(exit_code)
}

fn resolve_flags_and_init(args: Vec<std::ffi::OsString>) -> Result<deno_lib_ext::Flags, AnyError> {
    let flags = match flags_from_vec(args) {
        Ok(flags) => flags,
        Err(err @ clap::Error { .. }) if err.kind() == clap::error::ErrorKind::DisplayVersion => {
            // Ignore results to avoid BrokenPipe errors.
            deno_lib_ext::util::logger::init(None);
            let _ = err.print();
            std::process::exit(0);
        }
        Err(err) => {
            deno_lib_ext::util::logger::init(None);
            exit_for_error(AnyError::from(err))
        }
    };

    deno_lib_ext::util::logger::init(flags.log_level);

    // TODO(bartlomieju): remove in Deno v2.5 and hard error then.
    if flags.unstable_config.legacy_flag_enabled {
        println!(
            "⚠️  {}",
            (
                "The `--unstable` flag has been removed in Deno 2.0. Use granular `--unstable-*` flags instead.\nLearn more at: https://docs.deno.com/runtime/manual/tools/unstable_flags"
            )
    );
    }

    let default_v8_flags = match flags.subcommand {
        // Using same default as VSCode:
        // https://github.com/microsoft/vscode/blob/48d4ba271686e8072fc6674137415bc80d936bc7/extensions/typescript-language-features/src/configuration/configuration.ts#L213-L214
        deno_lib_ext::DenoSubcommand::Lsp => vec!["--max-old-space-size=3072".to_string()],
        _ => {
            // TODO(bartlomieju): I think this can be removed as it's handled by `deno_core`
            // and its settings.
            // deno_ast removes TypeScript `assert` keywords, so this flag only affects JavaScript
            // TODO(petamoriken): Need to check TypeScript `assert` keywords in deno_ast
            vec!["--no-harmony-import-assertions".to_string()]
        }
    };

    init_v8_flags(&default_v8_flags, &flags.v8_flags, get_v8_flags_from_env());
    // TODO(bartlomieju): remove last argument once Deploy no longer needs it
    deno_core::JsRuntime::init_platform(None, /* import assertions enabled */ false);

    Ok(flags)
}

fn exit_for_error(error: AnyError) -> ! {
    let mut error_string = format!("{error:?}");
    let mut error_code = 1;

    if let Some(e) = error.downcast_ref::<JsError>() {
        error_string = format_js_error(e);
    } else if let Some(deno_lib_ext::SnapshotFromLockfileError::IntegrityCheckFailed(e)) =
        error.downcast_ref::<deno_lib_ext::SnapshotFromLockfileError>()
    {
        error_string = e.to_string();
        error_code = 10;
    }

    exit_with_message(&error_string, error_code);
}

fn exit_with_message(message: &str, code: i32) -> ! {
    println!("{}: {}", "error", message.trim_start_matches("error: "));
    std::process::exit(code);
}

fn set_npm_user_agent() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| {
        std::env::set_var(
            deno_lib_ext::npm::NPM_CONFIG_USER_AGENT_ENV_VAR,
            deno_lib_ext::npm::get_npm_config_user_agent(),
        );
    });
}
