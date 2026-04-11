// FEAT-APP-001
// REQ-CORE-017

import { expect, test } from "@playwright/test";

const usesFailingWorkspace =
  process.env.SYU_APP_E2E_WORKSPACE?.includes("tests/fixtures/workspaces/failing") ?? false;

test("renders top tabs and linked spec content", async ({ page }) => {
  await page.goto("/");

  const topLevelSections = page.getByRole("navigation", { name: "Top level sections" });

  await expect(page.getByRole("heading", { name: /^syu/i })).toBeVisible();
  await expect(page.getByRole("button", { name: /^syu\b/i })).toBeVisible();
  await expect(topLevelSections.getByRole("button", { name: /^philosophy\b/i })).toBeVisible();
  await expect(topLevelSections.getByRole("button", { name: /^policies\b/i })).toBeVisible();
  await expect(topLevelSections.getByRole("button", { name: /^features\b/i })).toBeVisible();
  await expect(topLevelSections.getByRole("button", { name: /^requirements\b/i })).toBeVisible();
  await expect(page.getByText("Welcome to syu.")).toBeVisible();

  await page.getByRole("button", { name: "Dismiss welcome banner" }).click();
  await expect(page.getByText("Welcome to syu.")).toHaveCount(0);

  await page.reload();
  await expect(page.getByText("Welcome to syu.")).toHaveCount(0);

  await topLevelSections.getByRole("button", { name: /^features\b/i }).click();
  await page.getByRole("button", { name: /check\.yaml/i }).click();
  await expect(
    page.getByRole("heading", { name: /FEAT-CHECK-001 .* Unified validation command/i }),
  ).toBeVisible();
  await expect(page).toHaveURL(/#features\/FEAT-CHECK-001$/);
  await expect(page.getByText("SYU-workspace-load-001").first()).toBeVisible();

  await page.getByRole("button", { name: "REQ-CORE-001" }).click();
  await expect(
    page.getByRole("heading", {
      name: /REQ-CORE-001 .* Validate the linked specification graph with rule-backed diagnostics/i,
    }),
  ).toBeVisible();
  await expect(page).toHaveURL(/#requirements\/REQ-CORE-001$/);
  await expect(page.getByRole("button", { name: "← Back" })).toBeVisible();

  await page.getByRole("button", { name: "← Back" }).click();
  await expect(
    page.getByRole("heading", { name: /FEAT-CHECK-001 .* Unified validation command/i }),
  ).toBeVisible();
  await expect(page).toHaveURL(/#features\/FEAT-CHECK-001$/);
});

test("loads deep links and supports keyboard search navigation", async ({ page }) => {
  await page.goto("/#/requirements/REQ-CORE-001");

  await expect(
    page.getByRole("heading", {
      name: /REQ-CORE-001 .* Validate the linked specification graph with rule-backed diagnostics/i,
    }),
  ).toBeVisible();
  await expect(page).toHaveURL(/#requirements\/REQ-CORE-001$/);

  const searchInput = page.getByRole("searchbox", { name: "Search spec items" });
  await searchInput.fill("FEAT-CHECK-001");
  await searchInput.press("ArrowDown");
  await searchInput.press("ArrowUp");
  await searchInput.press("ArrowDown");
  await searchInput.press("Enter");

  await expect(
    page.getByRole("heading", { name: /FEAT-CHECK-001 .* Unified validation command/i }),
  ).toBeVisible();
  await expect(page).toHaveURL(/#features\/FEAT-CHECK-001$/);

  await searchInput.fill("no-such-result");
  await searchInput.press("ArrowUp");
  await searchInput.press("Enter");

  await expect(page.getByText("No items match.")).toBeVisible();
  await expect(
    page.getByRole("heading", { name: /FEAT-CHECK-001 .* Unified validation command/i }),
  ).toBeVisible();
});

test("keeps duplicate validation issues independently selectable", async ({ page }) => {
  test.skip(!usesFailingWorkspace, "requires the failing fixture workspace");

  await page.goto("/");

  const duplicateIssueRows = page.getByRole("button", { name: /SYU-trace-id-001/i });
  await expect(duplicateIssueRows).toHaveCount(2);
  const selectedIssue = page
    .getByRole("heading", { level: 3, name: "SYU-trace-id-001" })
    .locator("..");

  await duplicateIssueRows.nth(0).click();
  await expect(
    selectedIssue.getByText(
      "Declared implementation file `frontend/broken-feature.ts` does not mention `FEAT-FAIL-001`.",
    ),
  ).toBeVisible();
  await expect(selectedIssue.getByText("typescript:frontend/broken-feature.ts")).toBeVisible();

  await duplicateIssueRows.nth(1).click();
  await expect(
    selectedIssue.getByText(
      "Declared test file `src/broken_tests.rs` does not mention `REQ-FAIL-001`.",
    ),
  ).toBeVisible();
  await expect(selectedIssue.getByText("rust:src/broken_tests.rs")).toBeVisible();
});
