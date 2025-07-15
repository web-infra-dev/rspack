import propTypes from "prop-types";
import react from "react";
import reactDOM from "react-dom";

it("should load modules correctly", () => {
	expect(react).toBe("react");
	expect(reactDOM).toBe("react-dom");
	expect(propTypes).toBe("prop-types");
});
