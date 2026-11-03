import NodeFS from "node:fs";
import NodePath from "node:path";
const [_1, _2, file] = process.argv;

if (file && ["napi-binding.d.ts", "rspack.wasi.d.cts"].includes(NodePath.basename(file))) {
	const raw = getContent(NodeFS.readFileSync(file, "utf8"));
	const banner = getContent(
		NodeFS.readFileSync(NodePath.resolve(import.meta.dirname, "banner.d.ts"), "utf8")
	);
	const hasBOM = raw.hasBOM || banner.hasBOM;
	// Native builds preserve the WASI declaration. Replace its old banner even
	// when the handwritten types have changed since the last WASM build.
	const marker = "/* -- napi-rs generated below -- */";
	const markerIndex = raw.content.lastIndexOf(marker);
	const generated = markerIndex === -1
		? raw.content
		: raw.content.slice(markerIndex + marker.length).replace(/^\r?\n/, "");
	const content = (hasBOM ? "\uFEFF" : "") + banner.content + generated;
	NodeFS.writeFileSync(file, content);
}

/**
 *
 * @param {string} content
 */
function getContent(content) {
	const hasBOM = content.startsWith("\uFEFF");
	return {
		hasBOM,
		content: hasBOM ? content.slice(1) : content
	};
}
