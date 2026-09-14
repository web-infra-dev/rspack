export const tree = {
	deep: {
		path: {
			value: 42,
			read() {
				return this.value;
			}
		}
	}
};
export const nil = null;
export const keys = {
	"1000": "number",
	"10": "bigint",
	"true": "boolean",
	"null": "null",
	"/key/gi": "regexp",
	"a.b": "dot",
	"é": "unicode"
};
