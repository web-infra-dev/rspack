const TOP_OF_FILE = 1;

import { useParams } from "reexport-intermediate";

rs.mock("reexport-intermediate", () => {
	return {
		useParams: () => ({ id: "mocked-id" })
	};
});

it("should mock a re-exported module", () => {
	expect(useParams()).toEqual({ id: "mocked-id" });
});
