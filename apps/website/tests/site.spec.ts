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
    await expect(
      page.locator('a[href*="github.com/James-Jennison/quireforge"]'),
    ).toHaveCount(0);
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

test("keyboard and user media preferences retain an operable shell", async ({
  page,
}) => {
  await page.emulateMedia({ reducedMotion: "reduce" });
  await page.goto("/");

  await page.keyboard.press("Tab");
  const skipLink = page.getByRole("link", { name: "Skip to main content" });
  await expect(skipLink).toBeFocused();
  await expect(skipLink).toBeVisible();
  await page.keyboard.press("Enter");
  await expect(page.getByRole("main")).toBeFocused();

  const transitionDuration = await page
    .getByRole("link", { name: "Explore the roadmap" })
    .evaluate((element) => getComputedStyle(element).transitionDuration);
  const transitionDurationMs = transitionDuration.endsWith("ms")
    ? Number.parseFloat(transitionDuration)
    : Number.parseFloat(transitionDuration) * 1_000;
  expect(transitionDurationMs).toBeLessThanOrEqual(0.01);

  await page.emulateMedia({ forcedColors: "active" });
  await page.reload();
  const toggle = page.getByRole("button", { name: /use (dark|light) theme/i });
  await toggle.focus();
  const borderWidth = await toggle.evaluate(
    (element) => getComputedStyle(element).borderTopWidth,
  );
  expect(Number.parseFloat(borderWidth)).toBeGreaterThanOrEqual(2);
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
