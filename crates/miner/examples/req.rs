use std::{error::Error, fs};

use reqwest::Client;
use serde_json::{json, Value};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	// Read the JSON schema from file
	let schema: Value = serde_json::from_str(include_str!("../../../llm/schema.json"))?;

	let text = include_str!("../../../llm/gifts.txt").trim().split("\n\n");

	for text in text {
		let json = extract(&schema, text).await?;

		// Print the response
		println!("{}", serde_json::to_string_pretty(&json)?);
	}
	// println!("{}", response.text().await?);

	Ok(())
}

async fn extract(schema: &Value, text: &str) -> Result<Value, Box<dyn Error>> {
	// Construct the prompt
	let prompt = format!("Text: \n{}\n\nSchema: \n```json\n{}\n```\n\n", text, schema);

	// Create the JSON payload
	let payload = json!({
		"messages": [
			{
				"role": "system",
				"content": "You will receive some text and a JSON schema. Extract relevant information from the text and map it to the schema. Ensure the data types match the schema.",
			},
			{
				"role": "user",
				"content": prompt,
			}
		],
		"response_format": {
			"type": "json_schema",
			"json_schema": schema,
		},
		"stream": false,
	});

	// Send the POST request to the Ollama API
	let client = Client::new();
	let response = client
		.post("http://localhost:8080/v1/chat/completions")
		.json(&payload)
		.send()
		.await?;

	// Parse the response as JSON
	let response_json: Value = response.json().await?;
	// println!("{:#?}", response_json);
	let model_response = response_json["choices"][0]["message"]["content"]
		.as_str()
		.unwrap();
	let json: Value = serde_json::from_str(model_response)?;
	Ok(json)
}
