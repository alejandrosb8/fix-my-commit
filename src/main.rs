#![warn(clippy::all, clippy::pedantic, clippy::nursery)]

use bat::PrettyPrinter;
use clap::Parser;
use colored::Colorize;
use config::Config;
use question::{Answer, Question};
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, CONTENT_TYPE};
use serde_json::json;
use spinners::{Spinner, Spinners};
use std::process::Command;

mod config;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short = 'p', long = "prefix")]
    prefix: String,

    #[arg(short = 'm', long = "message")]
    message: String,

    /// Run the generated program without asking for confirmation
    #[arg(short = 'y', long)]
    force: bool,
}

const PREFIXES: [&str; 11] = [
    "Feat", "Fix", "Chore", "Docs", "Style", "Refactor", "Test", "Build", "Ci", "Perf", "Revert",
];

fn main() {
    let args = Cli::parse();
    let config = Config::new();

    let prefix = &args.prefix.to_lowercase();

    if string_in_array(&prefix, PREFIXES) {
        let client = Client::new();
        let mut spinner = Spinner::new(Spinners::BouncingBar, "Generating your message...".into());

        //datos de la petición
        let url = "https://api.cohere.ai/generate";
        //let cohere_version = "2022-12-06";
        let prompt = build_prompt(&prefix, &args.message);

        let mut headers = HeaderMap::new();
        headers.insert("Cohere-Version", HeaderValue::from_static("2022-12-06"));
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(
            "Authorization",
            HeaderValue::from_str(format!("Bearer {}", &config.api_key).as_str()).unwrap(),
        );

        let body = json!({
            "max_tokens": 40,
            "return_likelihoods": "NONE",
            "truncate": "END",
            "temperature": 0.5,
            "model": "xlarge",
            "k": 1,
            "p": 0,
            "frecuent_penalty": 0,
            "presence_penalty": 0,
            "stop_sequences": ["--"],
            "prompt": prompt,
        });

        let response = client
            .post(url)
            .headers(headers)
            .json(&body)
            .send()
            .unwrap();

        let status_code = response.status(); // extract the status code from the successful response

        if status_code.is_client_error() {
            let response_body = response
                .json::<serde_json::Value>() // extract the JSON data and deserialize it into a MyData struct
                .expect("Response error"); // panic with error message if there is one
            let error_message = response_body["error"]["message"].as_str().unwrap();
            spinner.stop_and_persist(
                "✖".red().to_string().as_str(),
                format!("API error: \"{error_message}\"").red().to_string(),
            );
            std::process::exit(1);
        } else if status_code.is_server_error() {
            spinner.stop_and_persist(
                "✖".red().to_string().as_str(),
                format!("Co:Here is currently experiencing problems. Status code: {status_code}")
                    .red()
                    .to_string(),
            );
            std::process::exit(1);
        }

        let old_message = response
            .json::<serde_json::Value>()
            .expect("Failed to parse response JSON")["generations"][0]["text"]
            .as_str()
            .unwrap_or_default()
            .trim()
            .to_string()
            .replace("--", "")
            .replace("\"", "");

        let new_message = format!("{}: {}", prefix, old_message).to_lowercase();

        spinner.stop_and_persist(
            "✔".green().to_string().as_str(),
            "This is your commit message".green().to_string(),
        );

        PrettyPrinter::new()
            .input_from_bytes(new_message.as_bytes())
            .language("bash")
            .grid(true)
            .print()
            .unwrap();

        let should_run = if args.force {
            true
        } else {
            Question::new(
                ">> Run the github commit? [Y/n]"
                    .bright_black()
                    .to_string()
                    .as_str(),
            )
            .yes_no()
            .until_acceptable()
            .default(Answer::YES)
            .ask()
            .expect("Couldn't ask question.")
                == Answer::YES
        };

        if should_run {
            let code = format!("git commit -m \"{}\"", new_message);

            config.write_to_history(code.as_str());
            spinner = Spinner::new(Spinners::BouncingBar, "Executing...".into());

            let output = Command::new("bash")
                .arg("-c")
                .arg(code.as_str())
                .output()
                .unwrap_or_else(|_| {
                    spinner.stop_and_persist(
                        "✖".red().to_string().as_str(),
                        "Failed to execute the commit.".red().to_string(),
                    );
                    std::process::exit(1);
                });

            if !output.status.success() {
                spinner.stop_and_persist(
                    "✖".red().to_string().as_str(),
                    "The commit threw an error.".red().to_string(),
                );
                println!("{}", String::from_utf8_lossy(&output.stderr));
                std::process::exit(1);
            }

            spinner.stop_and_persist(
                "✔".green().to_string().as_str(),
                "Commit done!".green().to_string(),
            );

            println!("{}", String::from_utf8_lossy(&output.stdout));
        }
    } else {
        println!("Invalid prefix");
    }
}

fn string_in_array(prefix: &str, array: [&str; 11]) -> bool {
    array
        .iter()
        .any(|&x| x.to_lowercase() == prefix.to_lowercase())
}

fn build_prompt(prefix: &str, message: &str) -> String {
    let format_prompt = String::from(
        "
This program write and fix a github commit message with the best practices (present simple verbs)

Examples of good messages:

Prefix: \"Feat\"
\"Add user authentication feature\"
\"Implement search functionality\"
\"Create responsive design for mobile devices\"
Prefix: \"Fix\"
\"Fix broken link in footer\"
\"Resolve server connection issue\"
\"Correct spelling errors in README file\"
Prefix: \"Refactor\"
\"Refactor code to improve performance\"
\"Simplify function logic for readability\"
\"Rename variable for clarity\"
Prefix: \"Docs\"
\"Update documentation with installation instructions\"
\"Add API documentation for endpoints\"
\"Remove outdated information from README file\"
Prefix: \"Test\"
\"Add unit tests for login functionality\"
\"Fix failing integration test for database connection\"
\"Update test suite for compatibility with new feature\"

Start program:

--
Prefix: \"Feat\"
Bad Message: \"added a button\"
Good Message: \"add button component\"
--
Prefix: \"Fix\"
Bad Message: \"fixed an navbar bug did not appears\"
Good Message: \"navbar does not appears\"
--
Prefix: \"Refactor\"
Bad Message: \"refactored the code for performance\"
Good Message: \"refactor code to improve performance\"
--
Prefix: \"Docs\"
Bad Message: \"updated the documentation\"
Good Message: \"update documentation with installation instructions\"
--
Prefix: \"Test\"
Bad Message: \"added a test for register\"
Good Message: \"add unit test for register functionality\"
--
Prefix: \"Feat\"
Bad Message: \"added a textinput\"
Good Message: \"add textinput component\"
--
Prefix: \"Refactor\"
Bad Message: \"refactored the code for better structure\"
Good Message: \"refactor code to improve code structure and readability\"
--
Prefix: \"Docs\"
Bad Message: \"added some notes to the documentation\"
Good Message: \"add explanatory notes to the documentation on how to use the API\"
--

",
    );
    format!(
        "{}Prefix: \"{}\"\nBad Message: \"{}\"\nGood Message: ",
        format_prompt, prefix, message
    )
}
