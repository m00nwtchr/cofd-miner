use std::fs;

use reqwest::Client;
use serde_json::{json, Value};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	// Read the JSON schema from file
	let schema_content = fs::read_to_string("schema.json")?;
	let schema: Value = serde_json::from_str(&schema_content)?;

	// Read the unstructured text from file
	let text_content = fs::read_to_string("gifts.txt")?;

	// Construct the prompt
	let prompt = format!(
        "The following is unstructured text and a known JSON schema. Extract relevant information from the text and map it to the schema. Ensure the data types match the schema.\n\nSchema: {}\n\nText: {}\n\nStructured JSON:",
        text_content.trim(),
        schema
    );

	// Create the JSON payload
	let payload = json!({
		"model": "llama3.2:3b",
		"messages": [
			{
				"role": "user",
				"content": prompt,
			}
		],
		// "prompt": prompt,
		"format": schema,
		"stream": false,
	});

	// Send the POST request to the Ollama API
	let client = Client::new();
	let response = client
		.post("http://localhost:11434/api/chat")
		.json(&payload)
		.send()
		.await?;

	// Parse the response as JSON
	let response_json: Value = response.json().await?;
	println!("{:#?}", response_json);
	let model_response = response_json["message"]["content"].as_str().unwrap();
	let json: Value = serde_json::from_str(model_response)?;

	// Print the response
	println!("{}", serde_json::to_string_pretty(&json)?);

	// println!("{}", response.text().await?);

	Ok(())
}
