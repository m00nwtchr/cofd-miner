#!/bin/bash

# Read the JSON schema
SCHEMA=$(cat schema.json)

# Read the unstructured text
TEXT=$(cat gifts.txt)

# Construct the prompt
PROMPT=$(
	cat <<EOF
The following is unstructured text and a known JSON schema. Extract relevant information from the text and map it to the schema. Ensure the data types match the schema.

Text: $TEXT

Schema: $SCHEMA

Structured JSON:
EOF
)

# Send the request to the Ollama API
RESPONSE=$(curl -s -X POST http://localhost:11434/api/chat \
	-H "Content-Type: application/json" \
	-d "$(jq -n \
		--arg model "llama3.2:1b" \
		--arg prompt "$PROMPT" \
		--argjson format "$SCHEMA" \
		'{
      model: $model,
      messages: [
        {
          role: "user",
          content: $prompt
        }
      ],
      format: $format
    }')")

# Print the response
echo "$RESPONSE"
