import { value as x } from './lib';

const call = value => {
	globalThis.__inlineExportImportBinding = value;
};

call(x);

export { x as a, x as b };
