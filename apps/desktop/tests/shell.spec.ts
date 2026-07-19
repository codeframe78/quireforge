import AxeBuilder from "@axe-core/playwright";
import { expect, test } from "@playwright/test";

test("desktop preview renders the honest semantic shell", async ({ page }) => {
  const response = await page.goto("/");

  expect(response?.ok()).toBe(true);
  await expect(
    page.getByRole("heading", { name: "A quiet place for ambitious work." }),
  ).toBeVisible();
  await expect(page.getByText("No project attached")).toBeAttached();
  await expect(
    page.getByText("Browser preview", { exact: true }),
  ).toBeAttached();
  await expect(
    page.getByText("Native probe unavailable").first(),
  ).toBeAttached();
  await expect(
    page.getByRole("heading", { name: "Authentication stays with Codex." }),
  ).toBeVisible();
  await expect(
    page.getByText("Native authentication unavailable"),
  ).toBeVisible();
  await expect(page.locator("main h1")).toHaveCount(1);

  const overflow = await page.evaluate(
    () => document.documentElement.scrollWidth - window.innerWidth,
  );
  expect(overflow).toBeLessThanOrEqual(1);
});

test("desktop preview has no automatically detectable accessibility violations", async ({
  page,
}) => {
  await page.goto("/");
  const results = await new AxeBuilder({ page }).analyze();

  expect(results.violations).toEqual([]);
});

test("theme control changes and persists the selected theme", async ({
  page,
}) => {
  await page.goto("/");
  const toggle = page.getByRole("button", { name: /use (dark|light) theme/iu });
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
