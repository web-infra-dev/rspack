import result from 'multi-alias';

it('should update alias resolution when a higher-priority file is created in watch mode', () => {
    switch (WATCH_STEP) {
        case "0":
            // Initial build:
            // `b.js` does not exist, so it should fall back to `a.js`.
            expect(result).toBe("a")
            break;
        case "1":
            // After update (b.js is created):
            // `b.js` now exists and has higher priority in the alias array.
            // The build should now resolve to `b.js`.
            expect(result).toBe("b")
            break;
        default:
            throw new Error('unexpected update');
    }
})
