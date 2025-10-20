import x from "./module";

it("should have correct this context in accept handler", async () => {
    expect(x).toEqual("ok1");
    let value;
    (function () {
        module.hot.accept("./module", () => {
            value = this;
        });
    }).call({ ok: true });

    await NEXT_HMR();
    expect(x).toEqual("ok2");
    expect(value).toEqual({ ok: true });
});
