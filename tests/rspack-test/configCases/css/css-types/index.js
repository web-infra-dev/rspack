import './style.css';
import * as style1 from './style1.local.css'
// import * as style2 from './style2.global.css'
import './style3.auto.css';
import * as style3 from './style4.modules.css'

it("should not parse css modules in type: css", () => {
    const style = getComputedStyle(document.body);
    expect(style.getPropertyValue("color")).toBe("rgb(255, 0, 0)");
    const links = Array.from(document.getElementsByTagName("link"));
    const css = [];

    // Skip first because import it by default
    for (const link of links.slice(1)) {
        css.push(getLinkSheet(link));
    }

    const cssString = css.join("\n");

    expect(cssString).toMatch(/\:local\(\.foo\)/);
    expect(cssString).toMatch(/\:global\(\.bar\)/);
});

it("should compile type: css/module", () => {
    const links = Array.from(document.getElementsByTagName("link"));
    const css = [];

    // Skip first because import it by default
    for (const link of links.slice(1)) {
        css.push(getLinkSheet(link));
    }

    const cssString = css.join("\n");

    expect(cssString).toContain(`.class2 {
    background: green; 
}`)
    expect(style1.class1).toBe('_style1_local_css-class1');
});

// MAYBE: support css/global
// it("should compile type: css/global", (done) => {
//     const element = document.createElement(".class3");
//     const style = getComputedStyle(element);
//     expect(style.getPropertyValue("color")).toBe(" red");
//     expect(style2.class4).toBe('_style2_global_css-class4');
//     done()
// });

it("should not parse css modules in type: css/auto", () => {
    const style = getComputedStyle(document.body);
    expect(style.getPropertyValue("background")).toBe("red");
    const links = Array.from(document.getElementsByTagName("link"));
    const css = [];

    // Skip first because import it by default
    for (const link of links.slice(1)) {
        css.push(getLinkSheet(link));
    }

    const cssString = css.join("\n");
    expect(cssString).toMatch(/\:local\(\.baz\)/);
    expect(cssString).toMatch(/\:global\(\.qux\)/);
});

it("should parse css modules in type: css/auto", () => {
    const links = Array.from(document.getElementsByTagName("link"));
    const css = [];

    // Skip first because import it by default
    for (const link of links.slice(1)) {
        css.push(getLinkSheet(link));
    }

    const cssString = css.join("\n");

    expect(cssString).toContain(`._style4_modules_css-class3 {
    color: red;
}`)
    expect(style3.class3).toBe('_style4_modules_css-class3');
});
