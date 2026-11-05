import path from 'node:path';

let times = 0;

export default async function () {
	times += 1;
	// the loader imports a different module on rebuild, so the
	// `importModule` call has to build a new module while the
	// rebuild is in progress
	const request = path.join(this.context, times === 1 ? 'lib1.js' : 'lib2.js');
	const exports = await this.importModule(request, {});
	return `export default ${JSON.stringify(exports.default)};`;
};
