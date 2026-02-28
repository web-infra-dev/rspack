it("should report missing modules if directory is not exist", () => {
	expect(() => require(`./lang/${lang}`)).toThrowError(
		/Cannot find module '.\/lang'/
	);
	expect(() => import(`./lang/${lang}`)).toThrowError(
		/Cannot find module '.\/lang'/
	);
});
