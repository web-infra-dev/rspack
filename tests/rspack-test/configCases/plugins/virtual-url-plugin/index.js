import { routes } from "virtual:routes"
import { app } from "virtual:app"
import json from "virtual:config"
import { ts } from "virtual:ts"
import txt from "virtual:txt"

it("should correctly load virtual modules with the js type.", async () => {
    expect(typeof routes.bar).toBe("function");
    expect(typeof routes.foo).toBe("function");
    expect(app).toBe("app");
    await Promise.all([
        routes.bar(),
        routes.foo()
    ]).then(([{bar}, {foo}]) => {
        expect(bar).toBe("bar");
        expect(foo).toBe("foo");
    });
});

it("should correctly load virtual modules with the json type.", () => {
    expect(json.name).toBe("virtual-url-plugin");
});

it("should correctly load virtual modules with the css type.", async () => {
  await import("virtual:style");
});

it("should correctly load virtual modules with the asset/source type.", () => {
    expect(txt).toBe("Hello world");
});

it("should correctly load virtual modules with custom loader.", () => {
    expect(ts).toBe("var");
});
