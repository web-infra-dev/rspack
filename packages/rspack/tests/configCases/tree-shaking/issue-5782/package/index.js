const aaa = {
	cc: {
		aa: 1,
		cc: undefined
	}
};

function bbb(a) {
	const { cc = aaa.cc.cc } = a;
	console.log("cc", cc);
}

export { bbb };
