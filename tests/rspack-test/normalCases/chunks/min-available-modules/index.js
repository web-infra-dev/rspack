import('./a');
if ((function () { return false; })()) {
    import('./d');
}
