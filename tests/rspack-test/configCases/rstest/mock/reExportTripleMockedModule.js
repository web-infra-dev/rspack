const TOP_OF_FILE = 1;

import { useParams } from "reexport-top";

rs.mock("reexport-top", () => {
	return {
		useParams: () => ({ id: "triple-mocked-id" })
	};
});

it("should mock A->B->C re-export chain", () => {
	expect(useParams()).toEqual({ id: "triple-mocked-id" });
});
