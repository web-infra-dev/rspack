// Emit CSS whose very last bytes are an unterminated `url(...)` so the tokenizer
// hits its end-of-input branch; a dropped final byte would truncate the request.
/** @type {import("@rspack/core").LoaderDefinition} */
export default function () {
	return '.a { background: url(./img.png';
}
