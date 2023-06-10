import fitz

def extract_text(fname):
	with fitz.open(fname) as doc:
		text = chr(12).join([page.get_text() for page in doc])
		return text