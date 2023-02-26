const triples = [
	{
		name: "x86_64-apple-darwin"
	},
	{
		name: "x86_64-unknown-linux-gnu",
		libc: "glibc"
	},
	{
		name: "x86_64-pc-windows-msvc"
	},
	{
		name: "aarch64-apple-darwin"
	},
	{
		name: "aarch64-unknown-linux-gnu",
		libc: "glibc"
	},
	{
		name: "armv7-unknown-linux-gnueabihf"
	},
	{
		name: "aarch64-unknown-linux-musl",
		libc: "musl"
	},
	{
		name: "x86_64-unknown-linux-musl",
		libc: "musl"
	}
];
const cpuToNodeArch = {
	x86_64: "x64",
	aarch64: "arm64",
	i686: "ia32",
	armv7: "arm"
};
const sysToNodePlatform = {
	linux: "linux",
	freebsd: "freebsd",
	darwin: "darwin",
	windows: "win32"
};
