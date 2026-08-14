export const a = "plain";
export let 原始拼接全局引用 = "local raw";
export let 转义拼接全局引用 = "local escaped";

if (globalThis.CHANGE_UNICODE_IDENTIFIER) {
	原始拼接全局引用 = "changed";
	转义拼接全局引用 = "changed";
}
