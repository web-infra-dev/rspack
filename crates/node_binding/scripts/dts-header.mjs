import NodeFS from "node:fs";
import NodePath from "node:path";
const [_1, _2, file] = process.argv;

if (file && ["napi-binding.d.ts", "rspack.wasi.d.cts"].includes(NodePath.basename(file))) {
	const raw = getContent(NodeFS.readFileSync(file));
	const banner = getContent(
		NodeFS.readFileSync(NodePath.resolve(import.meta.dirname, "banner.d.ts"))
	);
	const hasBOM = raw.hasBOM || banner.hasBOM;
	// Native builds preserve the existing WASI declaration, which already has its banner.
	const alreadyHasBanner = raw.buf
		.subarray(raw.offset, raw.offset + banner.buf.length - banner.offset)
		.equals(banner.buf.subarray(banner.offset));
	const content = Buffer.concat([
		hasBOM ? Buffer.from([0xfeff]) : Buffer.from([]),
		alreadyHasBanner ? Buffer.from([]) : banner.buf.subarray(banner.offset),
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
