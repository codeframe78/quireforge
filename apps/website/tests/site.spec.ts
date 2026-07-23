import AxeBuilder from "@axe-core/playwright";
import { expect, test } from "@playwright/test";

const routes = [
  "/",
  "/features/",
  "/integrations/",
  "/downloads/",
  "/installation/",
  "/documentation/",
  "/compatibility/",
  "/roadmap/",
  "/releases/",
  "/security/",
  "/contributing/",
  "/faq/",
  "/troubleshooting/",
  "/about/",
];

test("all public routes render their semantic shell", async ({ page }) => {
  for (const route of routes) {
    const response = await page.goto(route);
    expect(response?.ok(), `${route} should load`).toBe(true);
    await expect(page.locator("main h1")).toHaveCount(1);
    await expect(page.locator("footer")).toContainText(
      "unofficial community project",
    );
    await expect(page.locator('a[href*="github.com"]')).toHaveCount(0);
    const overflow = await page.evaluate(
      () => document.documentElement.scrollWidth - window.innerWidth,
    );
    expect(
      overflow,
      `${route} should not overflow horizontally`,
    ).toBeLessThanOrEqual(1);
  }
});

test("all public routes have no automatically detectable accessibility violations", async ({
  page,
}) => {
  for (const route of routes) {
    await page.goto(route);
    const results = await new AxeBuilder({ page }).analyze();
    expect(results.violations, `${route} should pass axe`).toEqual([]);
  }
});

test("theme control changes and persists the selected theme", async ({
  page,
}) => {
  await page.goto("/");
  const toggle = page.getByRole("button", { name: /use (dark|light) theme/i });
  const before = await page.locator("html").getAttribute("data-theme");
  await toggle.click();
  const after = await page.locator("html").getAttribute("data-theme");
  expect(after).not.toBe(before);
  await page.reload();
  await expect(page.locator("html")).toHaveAttribute(
    "data-theme",
    after ?? "dark",
  );
});
