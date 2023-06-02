it("css modules dedupe", () => {
	const style = require("./source.css");
	expect(style).toEqual({
		backButton:
			"source-css__backButton buttons-secondary-button-css__secondaryButton buttons-button-css__button",
		nextButton:
			"source-css__nextButton buttons-primary-button-css__primaryButton buttons-button-css__button"
	});
});
