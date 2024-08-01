it("should load the component from container", () => {
	return import("./App").then(({ default: App }) => {
		expect(App()).toBe("App rendered with [This is react 0.1.2]");
		const sharingReact = __webpack_require__.initializeSharingData.scopeToSharingDataMapping.default[0];
		// name: "react", version: "0", eager: 0, singleton: 1, requiredVersion: "*", strictVersion: 0
		expect(sharingReact.name).toBe("react");
		expect(sharingReact.version).toBe("0");
		expect(sharingReact.eager).toBe(0);
		expect(sharingReact.singleton).toBe(1);
		expect(sharingReact.requiredVersion).toBe("*");
		expect(sharingReact.strictVersion).toBe(0);
	});
});
