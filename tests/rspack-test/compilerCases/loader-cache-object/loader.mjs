import fs from 'node:fs';
import path from 'node:path';

export default function loaderCacheObjectLoader() {
	const { counterFile, depFile, depDir, extraFiles = [] } = this.getOptions();
	fs.mkdirSync(path.dirname(counterFile), { recursive: true });
	fs.appendFileSync(counterFile, 'x');

	const dep = depFile && fs.existsSync(depFile)
		? fs.readFileSync(depFile, 'utf8')
		: 'missing';
	const dir = depDir && fs.existsSync(depDir)
		? fs.readdirSync(depDir)
				.sort()
				.map(name => `${name}:${fs.readFileSync(path.join(depDir, name), 'utf8')}`)
				.join(',')
		: 'missing';
	const extra = extraFiles.map(file =>
		fs.existsSync(file) ? fs.readFileSync(file, 'utf8') : 'missing',
	);

	return `export default ${JSON.stringify({ mode: this.mode, dep, dir, extra })};`;
}
