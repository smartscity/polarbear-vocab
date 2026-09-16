import AxeBuilder from "@axe-core/playwright";
import { expect, test } from "@playwright/test";

const stories = ["home", "lexicon", "dataset-list", "dataset-detail", "listening", "session-setup", "study", "answer-result", "mistakes", "settings"] as const;
const viewports = [
  { name: "390x844", width: 390, height: 844 },
  { name: "430x932", width: 430, height: 932 },
  { name: "768x1024", width: 768, height: 1024 },
  { name: "1024x768", width: 1024, height: 768 },
  { name: "1280x800", width: 1280, height: 800 },
  { name: "1440x900", width: 1440, height: 900 },
  { name: "1728x1117", width: 1728, height: 1117 },
] as const;

for (const viewport of viewports) {
  for (const story of stories) {
    test(`${story} ${viewport.name}`, async ({ page }) => {
      await page.setViewportSize(viewport);
      await openStory(page, story);
      const overflow = await page.evaluate(() => document.documentElement.scrollWidth - document.documentElement.clientWidth);
      expect(overflow, "page must not overflow horizontally").toBeLessThanOrEqual(1);
      if (viewport.width < 640) await expectCompactTargets(page);
      await expect(page).toHaveScreenshot(`${story}-${viewport.name}.png`, { animations: "disabled", fullPage: true });
    });
  }
}

for (const story of stories) {
  test(`${story} has no serious accessibility violations`, async ({ page }) => {
    await page.setViewportSize({ width: 1280, height: 800 });
    await openStory(page, story);
    const results = await new AxeBuilder({ page }).withTags(["wcag2a", "wcag2aa", "wcag21aa"]).analyze();
    expect(results.violations.filter((violation) => ["serious", "critical"].includes(violation.impact ?? ""))).toEqual([]);
    await page.keyboard.press("Tab");
    const outlineWidth = await page.locator(":focus").evaluate((element) => getComputedStyle(element).outlineWidth);
    expect(Number.parseFloat(outlineWidth), "keyboard focus must be visible").toBeGreaterThanOrEqual(2);
  });

  test(`${story} supports dark appearance`, async ({ page }) => {
    await page.emulateMedia({ colorScheme: "dark", reducedMotion: "reduce" });
    await page.setViewportSize({ width: 1280, height: 800 });
    await openStory(page, story);
    await expect(page.locator("html")).toHaveAttribute("data-theme", "dark");
    await waitForDarkThemePaint(page);
    const results = await new AxeBuilder({ page }).withTags(["wcag2a", "wcag2aa", "wcag21aa"]).analyze();
    expect(results.violations.filter((violation) => ["serious", "critical"].includes(violation.impact ?? ""))).toEqual([]);
    await expect(page).toHaveScreenshot(`${story}-dark-1280x800.png`, { animations: "disabled", fullPage: true });
  });

  test(`${story} reflows at 200 percent text`, async ({ page }) => {
    await page.setViewportSize({ width: 390, height: 844 });
    await openStory(page, story);
    await page.locator("html").evaluate((element) => { element.style.fontSize = "200%"; });
    await page.evaluate(async () => {
      await document.fonts.ready;
      await new Promise<void>((resolve) => requestAnimationFrame(() => requestAnimationFrame(() => resolve())));
    });
    const overflow = await page.evaluate(() => document.documentElement.scrollWidth - document.documentElement.clientWidth);
    expect(overflow, "200 percent text must not cause horizontal page overflow").toBeLessThanOrEqual(1);
  });
}

test("listening exposes every offline voice style", async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 800 });
  await openStory(page, "listening");
  await page.getByRole("combobox", { name: "Voice style" }).click();
  await expect(page.getByRole("option")).toHaveText([
    "Male",
    "Female",
    "American English",
    "British English",
    "Hong Kong English",
    "Indian English",
    "Japanese English",
  ]);
});

async function waitForDarkThemePaint(page: import("@playwright/test").Page) {
  await expect.poll(() => page.locator("body").evaluate((element) => {
    const styles = getComputedStyle(element);
    const display = document.querySelector<HTMLElement>(".pb-display");
    return {
      background: styles.backgroundColor,
      color: styles.color,
      displayColor: display ? getComputedStyle(display).color : styles.color,
    };
  })).toEqual({
    background: "rgb(17, 23, 19)",
    color: "rgb(237, 243, 238)",
    displayColor: "rgb(237, 243, 238)",
  });
  await page.evaluate(() => new Promise<void>((resolve) => {
    requestAnimationFrame(() => requestAnimationFrame(() => resolve()));
  }));
}

async function openStory(page: import("@playwright/test").Page, story: string) {
  await page.goto(`/iframe.html?id=screens--${story}&viewMode=story`);
  await page.locator("#storybook-root > *").first().waitFor();
  await page.evaluate(() => document.fonts.ready);
}

async function expectCompactTargets(page: import("@playwright/test").Page) {
  const undersized = await page.locator("button:visible, input:visible, select:visible, [role=combobox]:visible").evaluateAll((elements) => elements.flatMap((element) => {
    const rectangle = element.getBoundingClientRect();
    return rectangle.width < 44 || rectangle.height < 44
      ? [{ tag: element.tagName, text: element.textContent?.trim(), width: rectangle.width, height: rectangle.height }]
      : [];
  }));
  expect(undersized, "compact interactive targets must be at least 44 by 44 pixels").toEqual([]);
}
