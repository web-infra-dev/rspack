it('should not panic', () => {
    import('./render').then(exports => {
        exports.render()
    })
})
