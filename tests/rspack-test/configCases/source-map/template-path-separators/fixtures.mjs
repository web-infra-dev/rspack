export const fixtures = [
	{
		source: String.raw`C:\project\src\index.ts?query=C:\query\value#fragment\value`,
		normalized: String.raw`C:/project/src/index.ts?query=C:\query\value#fragment\value`
	},
	{
		source: String.raw`\\server\share\index.ts?query=\query\value`,
		normalized: String.raw`//server/share/index.ts?query=\query\value`
	},
	{
		source: String.raw`\\?\C:\project\src\device.ts?query=\query\value`,
		normalized: String.raw`//?/C:/project/src/device.ts?query=\query\value`
	},
	{
		source: String.raw`/project/src/posix\name.ts?query=\query\value`,
		normalized: process.platform === "win32"
			? String.raw`/project/src/posix/name.ts?query=\query\value`
			: String.raw`/project/src/posix\name.ts?query=\query\value`
	},
	{
		source: String.raw`custom://host/path\name.ts?query=\query\value`,
		normalized: String.raw`custom://host/path\name.ts?query=\query\value`
	},
	{
		source: String.raw`custom+name.scheme://host/path\name.ts?query=\query\value`,
		normalized: String.raw`custom+name.scheme://host/path\name.ts?query=\query\value`
	},
	{
		source: String.raw`file:///C:/project/path\name.ts?query=\query\value`,
		normalized: String.raw`file:///C:/project/path\name.ts?query=\query\value`
	},
	{
		source: String.raw`C:\loader\index.js?option=\loader\value!C:\project\src\loaded.ts?query=\query\value`,
		absolute: String.raw`C:\project\src\loaded.ts?query=\query\value`,
		normalized: String.raw`C:/project/src/loaded.ts?query=\query\value`
	},
	{
		source: "C:\\project\\src\\escaped\u200b#name\\index.ts?query=\\query\\value",
		normalized: "C:/project/src/escaped\u200b#name/index.ts?query=\\query\\value"
	},
	{
		source: String.raw`builtin:swc-loader?option=\loader\value!Z:\project\src\builtin.ts?query=\query\value`,
		absolute: String.raw`Z:\project\src\builtin.ts?query=\query\value`,
		normalized: String.raw`Z:/project/src/builtin.ts?query=\query\value`
	}
];
