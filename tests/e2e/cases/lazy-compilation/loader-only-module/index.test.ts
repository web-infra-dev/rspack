import { expect, test } from "@/fixtures";

test("lazy compilation handles loader-only entries without resource paths", async ({ page }) => {
    // Wait for the lazy-compiled content to load
    const actionButton = page.getByRole("button", { name: "Click me" });
    await expect(actionButton).toBeVisible({ timeout: 1000 });

    // Test that the lazy module functions correctly
    await actionButton.click();

    // Verify successful execution
    await expect(page).toHaveURL(/success$/);
});
