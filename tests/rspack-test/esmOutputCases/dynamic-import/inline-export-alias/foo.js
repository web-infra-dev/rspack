const call = value => {
	globalThis.__inlineExportAlias = value;
};

const x = 1;

call(x);

export { x as a, x as b };
