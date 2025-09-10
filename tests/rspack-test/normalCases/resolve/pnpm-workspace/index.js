it('resolve recursive symbol link in pnpm workspace', async () => {
    const name = (await import('./packages/app')).default;
    expect(name).toBe('react');
})