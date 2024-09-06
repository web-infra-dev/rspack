it("should load the component from container", () => {
	return import("./App").then(({ default: App }) => {
		expect(App()).toBe("App rendered with [This is react 0.1.2]");
		const sharingReact = __webpack_require__.initializeSharingData.scopeToSharingDataMapping.default[0];
		// name: "react", version: "0.1.2", eager: 0, singleton: 1, requiredVersion: "*", strictVersion: 0
		expect(sharingReact.name).toBe("react");
		expect(sharingReact.version).toBe("0.1.2");
		expect(sharingReact.eager).toBe(0);
		expect(sharingReact.singleton).toBe(1);
		expect(sharingReact.requiredVersion).toBe("*");
		expect(sharingReact.strictVersion).toBe(0);
		const initOptionsReact = __webpack_require__.federation.initOptions.shared.react[0];
		const initOptionsReactShareConfig = __webpack_require__.federation.initOptions.shared.react[0].shareConfig;
		expect(initOptionsReact.version).toBe("0.1.2");
		expect(initOptionsReactShareConfig.eager).toBe(0);
		expect(initOptionsReactShareConfig.singleton).toBe(1);
		expect(initOptionsReactShareConfig.requiredVersion).toBe("*");
		expect(initOptionsReactShareConfig.strictVersion).toBe(0);
	});
});
