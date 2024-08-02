it("should not output asset when emit is false", () => {
    expect(url).toEqual("images/file.png");
    expect(url2).toEqual("images/file.jpg");

    expect(fs.existsSync(path.join(__STATS__.outputPath, url))).toBe(false);
    expect(fs.existsSync(path.join(__STATS__.outputPath, url2))).toBe(true);
});
