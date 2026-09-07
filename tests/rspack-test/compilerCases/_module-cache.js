const fs = require("node:fs");
const path = require("node:path");

let timestamp = Date.now() - 60_000;

exports.write = (root, name, content) => {
	const file = path.join(root, name);
	fs.mkdirSync(path.dirname(file), { recursive: true });
	fs.writeFileSync(file, content);
	// Keep timestamps distinct and outside the snapshot's filesystem accuracy window.
	const time = new Date((timestamp += 10));
	fs.utimesSync(file, time, time);
	return file;
};

exports.close = (compiler) =>
	new Promise((resolve, reject) => {
		compiler.close((error) => (error ? reject(error) : resolve()));
	});
