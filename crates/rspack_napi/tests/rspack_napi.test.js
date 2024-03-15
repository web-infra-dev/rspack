const { eqByRef, tsfnRefCallInMain, tsfnRefCallInAnotherThread } = require('..');

describe("ByRef", () => {
    it("should use reference equality", () => {
        const func = () => { };
        expect(eqByRef(func, func)).toBe(true);
        expect(eqByRef(func, () => { })).toBe(false);
    })
    it("should throw if value type is passed", () => {
        expect(() => { eqByRef(1, 1) }).toThrow("Invalid value used as weak map key");
    })
});

describe("ThreadSafeFunctionWithRef", () => {
    describe("call_sync", () => {
        it("can be called in the main thread", () => {
            expect(tsfnRefCallInMain(n => n * 2, 21)).toBe(42);
        });
        it("should throw if return type is incorrect when called in the main thread", () => {
            expect(() => tsfnRefCallInMain(() => "", 0)).toThrow("Failed to convert");
        });

        it("rethrows the error with the error message when called in the main thread", () => {
            expect(() => tsfnRefCallInMain(() => {
                throw new Error("test error message")
            }, 21)).toThrow("test error message")
        });


        it("should throw if return type is incorrect when called in a different thread", () => {
            expect(() => tsfnRefCallInAnotherThread(() => "", 0)).rejects.toThrow("Failed to convert");
        });

        it("can be called in a different thread", async () => {
            expect(await tsfnRefCallInAnotherThread(n => n * 2, 21)).toBe(42);
        });

        it("rethrows the error with the error message when called in a different thread", () => {
            expect(() => tsfnRefCallInAnotherThread(() => {
                throw new Error("test error message")
            }, 21)).rejects.toThrow("test error message")
        })
    })
})
