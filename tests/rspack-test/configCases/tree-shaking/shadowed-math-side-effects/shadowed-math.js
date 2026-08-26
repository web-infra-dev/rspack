const Math = {
	run() {
		globalThis.__SHADOWED_MATH_CALLED__ = true;
	}
};

Math.run();
