const useDynamicRouteParams = typeof window === "undefined" ? require("./server").useDynamicRouteParams : undefined;

export function usePathname() {
	useDynamicRouteParams == null ? void 0 : useDynamicRouteParams("usePathname()");
	return "/";
}
