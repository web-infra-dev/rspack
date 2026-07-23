import disabled from './disabled';

it("should work import.meta.env with EnvironmentPlugin", () => {
    expect(import.meta.env.AAA).toBe(process.env.AAA);
});

it("should preserve import.meta.env when its parser property is disabled", () => {
    expect(disabled.direct.RUNTIME).toBe('runtime');
    expect(disabled.destructured).toBe(disabled.direct);
    expect(disabled.defined).toBe(undefined);
    expect(disabled.destructuredDefined).toBe(undefined);
    expect(disabled.definedType).toBe('undefined');
    expect(disabled.type).toBe('object');
});

it("import.meta.env behaves like process.env", () => {
    try {
        const importMetaEnv = import.meta.env;
        importMetaEnv;
        const processEnv = process.env;
        processEnv;
        const UNKNOWN_PROPERTY = import.meta.env.UNKNOWN_PROPERTY;
        UNKNOWN_PROPERTY;
        const UNKNOWN_PROPERTY_2 = process.env.UNKNOWN_PROPERTY_2;
        UNKNOWN_PROPERTY_2;
        typeof import.meta.env;
        typeof process.env;

        const { env } = import.meta;
        env;
    } catch (_e) {
        // ignore
    }
});
