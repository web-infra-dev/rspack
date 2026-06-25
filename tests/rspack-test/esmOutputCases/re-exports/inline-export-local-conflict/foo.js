const call = value => {
	globalThis.__inlineExportLocalConflict = value;
};

const foo = 'foo';

call(foo);

export { foo };
