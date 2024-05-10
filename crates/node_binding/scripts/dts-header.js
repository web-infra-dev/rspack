const NodeFS = require("node:fs");
const NodePath = require("node:path");
const [_1, _2, file] = process.argv;

if (file && NodePath.basename(file) === "binding.d.ts") {
	const raw = getContent(NodeFS.readFileSync(file));
	const banner = getContent(
		NodeFS.readFileSync(NodePath.resolve(__dirname, "banner.d.ts"))
	);
	const hasBOM = raw.hasBOM || banner.hasBOM;
	const content = Buffer.concat([
		hasBOM ? Buffer.from([0xfeff]) : Buffer.from([]),
		banner.buf.subarray(banner.offset),
		raw.buf.subarray(raw.offset)
	]);
	NodeFS.writeFileSync(file, content);
}

/**
 *
 * @param {Buffer} buf
 */
function getContent(buf) {
	const hasBOM = buf[0] === 0xfeff;
	return {
		hasBOM,
		offset: hasBOM ? 1 : 0,
		buf
	};
}
