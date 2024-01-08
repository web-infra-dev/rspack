const foo = {
	noop: () => {},
	rename: 114,
	otherFields: 514
};

export const { noop, rename: reassign, ...rest } = foo;
export const [item1, ...items] = [1, 1, 4, 5, 1, 4];
