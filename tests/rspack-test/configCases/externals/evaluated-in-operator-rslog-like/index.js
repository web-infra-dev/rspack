import { logger } from "rslog";

it("should keep evaluated `in` operator imports bound for rslog-like packages", () => {
	expect(logger).toBeDefined();
	expect(typeof logger.info).toBe("function");
});
