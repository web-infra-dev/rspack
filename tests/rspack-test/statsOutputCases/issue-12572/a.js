export function a() {
    import('./b').then(({ b }) => {
        b();
    });
}
