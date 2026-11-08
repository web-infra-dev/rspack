export const tree = {
	a: {
		b: {
			value: 42,
			read() {
				return this.value;
			}
		}
	}
};
export const missing = null;
export const build = () => tree;
