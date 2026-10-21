export default function (source) {
	return Buffer.from(`\ufeff${source}`);
}
