import React from 'react';

const element = <div></div>;

it("react classic", () => {
	expect(element.type).toBe("div");
});
