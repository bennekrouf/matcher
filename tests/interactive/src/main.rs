use colored::*;
use run_testcase::TestCase;

mod run_testcase;
use crate::run_testcase::run_test_case;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let host = std::env::var("HOST").unwrap_or_else(|_| "http://localhost:50030".to_string());
    println!("{}", "Starting interactive matcher service tests...".blue());
    println!("Using server at {}\n", host);

    let test_cases = vec![
        TestCase {
            query: "envoie le document par email à fawzan@gmail.com".to_string(),
            language: "fr".to_string(),
            description: "Basic email sending test".to_string(),
            confirm: true,
            parameter_values: vec!["subject".to_string(), "body".to_string()],
        },
        TestCase {
            query: "send the document".to_string(),
            language: "en".to_string(),
            description: "Email with missing parameters".to_string(),
            confirm: true,
            parameter_values: vec![
                "john@example.com".to_string(),
                "Test Subject".to_string(),
                "Test Body".to_string(),
            ],
        },
        TestCase {
            query: "envoi le rapport".to_string(),
            language: "fr".to_string(),
            description: "Test with cancellation".to_string(),
            confirm: false,
            parameter_values: vec![],
        },
    ];

    for test_case in test_cases {
        run_test_case(&host, test_case).await?;
    }

    println!("\n{}", "All tests completed.".green());
    Ok(())
}
