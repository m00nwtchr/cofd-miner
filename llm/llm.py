def generate_facet_json(input_text, json_schema):
    prompt = f"""
You are an AI assistant trained to convert structured text into JSON format.

Here is the provided JSON schema:
{json_schema}

Now, based on this schema, convert the following input text into JSON format:

Input Text:
{input_text}

Ensure the output follows the structure defined in the schema. If any required fields are missing in the input text, try to infer them from the context. Please return a well-formatted JSON object with keys and values corresponding to the schema.
"""

    return prompt

from transformers import LlamaForCausalLM, LlamaTokenizer
import torch

# Load the LLaMA 13B model and tokenizer
model_name = "meta/llama-13b"  # You might need to change this based on the actual model path

tokenizer = LlamaTokenizer.from_pretrained(model_name)
model = LlamaForCausalLM.from_pretrained(model_name)

# Set the device (GPU if available, otherwise CPU)
device = "cuda" if torch.cuda.is_available() else "cpu"
model.to(device)

# Function to generate a JSON-like structure from input text
def generate_facet_json(text):
    # Tokenize the input text
    inputs = tokenizer(text, return_tensors="pt").to(device)

    # Generate the output
    with torch.no_grad():
        outputs = model.generate(inputs["input_ids"], max_length=512, num_return_sequences=1)

    # Decode the generated text
    decoded_output = tokenizer.decode(outputs[0], skip_special_tokens=True)

    # Return the decoded output (formatted as JSON)
    return decoded_output

# Define the input text (Facet description)
input_text = """
COLD EMBRACE (CUNNING)
With this Facet, the Uratha embraces the chill of the grave to still the beat of her heart. Her flesh grows cold. To all intents and purposes, she appears dead.
Cost: 1 Essence
Dice Pool: Stamina + Medicine + Cunning
Action: Instant
Duration: 1 hour per success
Roll Results
Dramatic Failure: The Uratha brings herself too close to death, and suffers one point of aggravated damage.
Failure: The Uratha fails to still her body's vital signs.
Success: The Uratha successfully stills her body's vital signs. She appears to be freshly dead, displaying no pulse or respiration, and her natural regeneration ceases for the duration. Wounds clot and the body displays all the signs of being a corpse.
Exceptional Success: The Uratha's morbid flesh also becomes more resilient, reducing all damage she suffers by one.
"""

# Define the JSON schema
json_schema = """
{
	"$schema": "https://json-schema.org/draft/2020-12/schema",
	"title": "Facets",
	"type": "array",
	"items": {
		"title": "Facet",
		"type": "object",
		"properties": {
			"name": {
				"type": "string"
			},
			"renown": {
				"title": "Renown",
				"type": "string",
				"enum": [
					"Purity",
					"Glory",
					"Honor",
					"Wisdom",
					"Cunning"
				]
			},
			"description": {
				"type": "string"
			},
			"effects": {
				"type": "string"
			},
			"cost": {
				"type": "string"
			},
			"dicePool": {
				"type": "string"
			},
			"action": {
				"type": "string"
			},
			"duration": {
				"type": "string"
			},
			"rollResults": {
				"title": "Roll Results",
				"type": "object",
				"properties": {
					"exceptionalSuccess": {
						"title": "Exceptional Success",
						"type": "string"
					},
					"success": {
						"title": "Success",
						"type": "string"
					},
					"failure": {
						"title": "Failure",
						"type": "string"
					},
					"dramaticFailure": {
						"title": "Dramatic Failure",
						"type": "string"
					}
				},
				"required": [
					"success",
					"failure"
				]
			}
		},
		"required": [
			"name",
			"renown"
		]
	},
	"minItems": 1,
	"maxItems": 5
}
"""

# Generate the prompt
prompt = generate_facet_json(input_text, json_schema)

# Get the generated output
json_output = generate_facet_json(prompt)
print(json_output)
