use std::io::Write;

use anyhow::Result;
use cofd_schema::item::gift::Other;
use itertools::Either;
use mistralrs::{
	llguidance::JsonCompileOptions, GgufModelBuilder, IsqType, PagedAttentionMetaBuilder,
	RequestBuilder, Response, TextMessageRole, TextModelBuilder,
};
use serde_json::{json, Value};

#[tokio::main]
#[cfg(feature = "llm-parser")]
async fn main() -> Result<()> {
	let model = GgufModelBuilder::new(
		"bartowski/Llama-3.2-1B-Instruct-GGUF",
		vec!["Llama-3.2-1B-Instruct-Q4_K_M.gguf"],
	)
	.with_tok_model_id("meta-llama/Llama-3.2-1B-Instruct")
	.with_paged_attn(|| PagedAttentionMetaBuilder::default().build())?
	.with_logging()
	.build()
	.await?;

	// let schema = schemars::schema_for!(cofd_schema::item::gift::Gift<Other>);
	// let mut json = schema.as_value().clone();

	let schema = include_str!("../../../llm/schema.json");
	let json: Value = serde_json::from_str(schema)?;

	let text = include_str!("../../../llm/gifts.txt");

	let prompt =
		format!(
			"The following is unstructured text and a known JSON schema. Extract relevant information from the text and map it to the schema. Ensure the data types match the schema.\n\nText: {}\n\nSchema: {}\n\nStructured JSON:",
			text,
			json
		);
	let tokens = model
		.tokenize(Either::Right(prompt.clone()), None, true, true)
		.await?;

	let request = RequestBuilder::new()
		.set_constraint(mistralrs::Constraint::JsonSchema(json))
		.set_sampler_max_len(tokens.len() * 2)
		.add_message(TextMessageRole::User, prompt)
		.set_sampler_temperature(0.5);

	let response = model.send_chat_request(request).await?;
	//
	// let mut stream = model.stream_chat_request(request).await?;
	//
	// println!("ABC");
	//
	// let stdout = std::io::stdout();
	// let lock = stdout.lock();
	// let mut buf = std::io::BufWriter::new(lock);
	// while let Some(chunk) = stream.next().await {
	// 	if let Response::Chunk(chunk) = chunk {
	// 		buf.write_all(chunk.choices[0].delta.content.as_bytes())?;
	// 	} else {
	// 		println!("Response: {:?}", chunk.as_result());
	// 		// Handle errors
	// 	}
	// }

	println!(
		"{}",
		response.choices[0].message.content.as_ref().unwrap().trim()
	);

	Ok(())
}
