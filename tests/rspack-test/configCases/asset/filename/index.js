import png from "../_images/file.png";
import png1 from "../_images/file.png?custom1";
import png2 from "../_images/file.png?custom2";
import jpeg2 from "../_images/file.jpg?custom2";
import png3 from "../_images/file.png?custom3";

it("should change filenames", () => {
	expect(png).toEqual("images/failure.png");
	expect(png1).toEqual("custom-images/success1.png");
	expect(png2).toEqual("custom-images/success2.png");
	expect(jpeg2).toEqual("images/failure2.jpg");

	const match = png3.match(
		/^模板\/file-([a-f0-9]{8})-([a-f0-9]{16})-([a-f0-9]{8})-\[contenthash:o\]-([a-f0-9]+)-(.{4})\.png$/,
	);
	expect(match).not.toBeNull();
	expect(match[1]).toBe(match[3]);
	expect(match[2].startsWith(match[1])).toBe(true);
	expect(match[4].length).toBeGreaterThan(0);
	expect(match[5]).toHaveLength(4);
});
