import png from "../_images/file.png";
import png1 from "../_images/file.png?custom1";
import png2 from "../_images/file.png?custom2";
import jpeg2 from "../_images/file.jpg?custom2";

it("should change filenames", () => {
	expect(png).toEqual("images/failure.png");
	expect(png1).toEqual("custom-images/success1.png");
	expect(png2).toEqual("custom-images/success2.png");
	expect(jpeg2).toEqual("images/failure2.jpg");
});
