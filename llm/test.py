import spacy
from spacy.tokens import DocBin

# Load or train a spaCy model
nlp = spacy.load("en_core_web_sm")

# Example unstructured text
text = """The "Shadow Bind" facet represents Glory. It costs 2 Essence to activate 
and lasts one scene. The roll is Presence + Occult. On success, the shadow grapples 
the target. On failure, the user is disoriented."""

# Process text
doc = nlp(text)

# Extract entities
entities = {}
for ent in doc.ents:
    if ent.label_ == "FACET_NAME":  # Define custom labels during training
        entities["name"] = ent.text
    elif ent.label_ == "RENOWN":
        entities["renown"] = ent.text
    # Add logic for other entities

print(entities)

