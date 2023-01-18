import { createFsFromVolume, Volume } from "memfs";
import { rspack } from "..";

describe("MultiStats", () => {
	it("should create JSON of children stats", done => {
		const compiler = rspack([
			{
				context: __dirname,
				entry: "./fixtures/a"
			},
			{
				context: __dirname,
				entry: "./fixtures/b"
			}
		]);
		compiler.outputFileSystem = createFsFromVolume(new Volume());
		compiler.run((err, stats) => {
			if (err) return done(err);
			try {
				const statsObject = stats!.toJson();
				expect(statsObject).toEqual(
					expect.objectContaining({ children: expect.any(Array) })
				);
				expect(statsObject.children).toHaveLength(2);
				done();
			} catch (e) {
				done(e);
			}
		});
	});
});
