it("basic", () => {
	const style = require("./source.css");
	expect(style).toEqual({
		backButton:
			"backButton-source.css secondaryButton-buttons/secondary-button.css button-buttons/button.css ",
		nextButton:
			"nextButton-source.css primaryButton-buttons/primary-button.css button-buttons/button.css "
	});
});
