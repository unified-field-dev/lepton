import { test, expect, authDialog, dismissAuthOverlay } from "./fixtures";

async function gotoAccountSettingsSignedIn(
  page: import("@playwright/test").Page,
  auth: {
    signIn: (opts: {
      email: string;
      password: string;
      referer?: string;
    }) => Promise<void>;
  },
  email: string,
  password: string,
) {
  await auth.signIn({ email, password, referer: "/welcome" });
  await expect(page).toHaveURL(/\/welcome/, { timeout: 60_000 });
  await expect(page.getByTestId("welcome-authenticated")).toBeVisible();
  await page.goto("/user/account-settings");
  await expect(page.getByTestId("account-settings-container")).toBeVisible({
    timeout: 60_000,
  });
  await expect(page.getByTestId("connected-accounts-section")).toBeVisible();
}

async function logout(page: import("@playwright/test").Page) {
  await page.goto("/auth/logout");
  await expect(page.getByTestId("logout-container")).toBeVisible({
    timeout: 60_000,
  });
  await authDialog(page)
    .getByTestId("logout-button")
    .getByRole("button")
    .click();
  await expect(page.getByTestId("user-menu-signin")).toBeVisible({
    timeout: 60_000,
  });
}

test.describe("pw-oauth link settings", () => {
  // Shared mock IdP subjects (`mock:google:mock-code` / `mock:github:mock-code`) —
  // run serially so claims do not race across workers.
  test.describe.configure({ mode: "serial" });

  test("pw-oauth-link-google-happy", async ({ page, auth }) => {
    const email = `oauth-link-${Date.now()}@example.com`;
    const password = "CorrectHorseBattery1!";
    await gotoAccountSettingsSignedIn(page, auth, email, password);

    await expect(page.getByTestId("connected-accounts-empty")).toBeVisible();
    await page
      .getByTestId("connected-accounts-link-google")
      .getByRole("button")
      .click();

    await expect(page).toHaveURL(/\/user\/account-settings/, {
      timeout: 60_000,
    });
    await expect(page.getByTestId("connected-accounts-section")).toBeVisible({
      timeout: 60_000,
    });
    await expect(page.getByTestId("connected-accounts-row")).toBeVisible({
      timeout: 30_000,
    });
    await expect(page.getByTestId("connected-accounts-row")).toContainText(
      /Google/i,
    );

    await dismissAuthOverlay(page);
    const unlinkBtn = page
      .getByTestId("connected-accounts-row")
      .getByRole("button", { name: "Unlink" });
    try {
      await unlinkBtn.click({ timeout: 5_000 });
    } catch {
      await unlinkBtn.click({ force: true, timeout: 15_000 });
    }
    await expect(
      page.getByTestId("connected-accounts-unlink-confirm"),
    ).toBeVisible();
    await page
      .getByTestId("connected-accounts-unlink")
      .getByRole("button")
      .click({ force: true });

    await expect(page.getByTestId("connected-accounts-success")).toBeVisible({
      timeout: 30_000,
    });
    await expect(page.getByTestId("connected-accounts-empty")).toBeVisible({
      timeout: 30_000,
    });
  });

  test("pw-oauth-link-account-taken-sad", async ({ page, auth }) => {
    // Claim the shared mock Google subject with a password user (leave it linked).
    const ownerEmail = `oauth-owner-${Date.now()}@example.com`;
    const password = "CorrectHorseBattery1!";
    await gotoAccountSettingsSignedIn(page, auth, ownerEmail, password);
    await page
      .getByTestId("connected-accounts-link-google")
      .getByRole("button")
      .click();
    await expect(page).toHaveURL(/\/user\/account-settings/, {
      timeout: 60_000,
    });
    await expect(page.getByTestId("connected-accounts-row")).toBeVisible({
      timeout: 30_000,
    });
    await logout(page);

    const otherEmail = `oauth-taken-${Date.now()}@example.com`;
    await gotoAccountSettingsSignedIn(page, auth, otherEmail, password);
    await page
      .getByTestId("connected-accounts-link-google")
      .getByRole("button")
      .click();

    await expect(page.getByTestId("oauth-callback-container")).toBeVisible({
      timeout: 60_000,
    });
    await expect(page.getByTestId("oauth-callback-error")).toBeVisible({
      timeout: 30_000,
    });
    await expect(page.getByTestId("oauth-callback-error")).toContainText(
      /already linked to another user/i,
    );
  });

  test("pw-oauth-unlink-last-method-sad", async ({ page }) => {
    await page.goto("/auth/signup?referer=/welcome");
    await expect(page.getByTestId("signup-container")).toBeVisible({
      timeout: 60_000,
    });
    await authDialog(page)
      .getByTestId("oauth-continue-github")
      .getByRole("button")
      .click();
    await expect(page).toHaveURL(/\/welcome(?:\?|$)/, { timeout: 60_000 });
    await expect(page.getByTestId("welcome-authenticated")).toBeVisible({
      timeout: 60_000,
    });
    // OAuth callback may still show "Finishing sign-in…" — wait it out before navigating.
    await dismissAuthOverlay(page);
    await expect(page.getByRole("dialog")).toHaveCount(0, { timeout: 30_000 });

    await page.goto("/user/account-settings");
    await expect(page.getByTestId("connected-accounts-section")).toBeVisible({
      timeout: 60_000,
    });
    await expect(page.getByTestId("connected-accounts-row")).toBeVisible({
      timeout: 30_000,
    });

    await dismissAuthOverlay(page);
    const unlinkBtn = page
      .getByTestId("connected-accounts-row")
      .getByRole("button", { name: "Unlink" });
    try {
      await unlinkBtn.click({ timeout: 5_000 });
    } catch {
      await unlinkBtn.click({ force: true, timeout: 15_000 });
    }
    await expect(
      page.getByTestId("connected-accounts-unlink-confirm"),
    ).toBeVisible();
    await page
      .getByTestId("connected-accounts-unlink")
      .getByRole("button")
      .click({ force: true });

    await expect(page.getByTestId("connected-accounts-error")).toBeVisible({
      timeout: 30_000,
    });
    await expect(page.getByTestId("connected-accounts-error")).toContainText(
      /keep at least one way to sign in/i,
    );
    await expect(page.getByTestId("connected-accounts-row")).toBeVisible();
  });
});
