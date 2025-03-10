import {test, expect} from "@/fixtures";

test("should inject innerHTML for added asset tags", async ({
    page
}) => {
    const scripts = await page.$$("script[id=inner-html-tag]");

    for (const script of scripts) {
        const innerHtml = await script.innerHTML();
        expect(innerHtml).toEqual('console.log("injected source code");');
    }
});
