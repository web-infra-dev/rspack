import externalReact from "react";
import { React as localReact } from "./local";

it("should keep generated external globals out of the concatenated local scope", () => {
	expect(externalReact.version).toBe("global");
	expect(localReact.version).toBe("local");
});
