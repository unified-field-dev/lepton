import {
  test,
  expect,
  clickTestId,
} from "./fixtures";

async function fillWipeForm(
  page: import("@playwright/test").Page,
  phrase: string,
  password: string,
) {
  const form = page.getByTestId("account-wipe-form");
  const phraseInput = form
    .getByTestId("account-wipe-confirm-phrase")
    .locator("input");
  const passwordInput = form
    .getByTestId("account-wipe-current-password")
    .locator("input");
  await expect(async () => {
    await phraseInput.fill(phrase);
    await passwordInput.fill(password);
    await expect(phraseInput).toHaveValue(phrase);
    await expect(passwordInput).toHaveValue(password);
  }).toPass({ timeout: 15_000 });
}

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
  await expect(page.getByTestId("account-wipe-form")).toBeVisible();
}

test.describe("pw-account-wipe", () => {
  test("pw-account-wipe-bad-password", async ({ page, auth }) => {
    const email = `wipe-bad-${Date.now()}@example.com`;
    const password = "CorrectHorseBattery1!";
    await gotoAccountSettingsSignedIn(page, auth, email, password);

    await fillWipeForm(page, "DELETE", "WrongPassword!!!!1");
    await clickTestId(page, "account-wipe-submit");

    await expect(page.getByTestId("account-wipe-error")).toBeVisible({
      timeout: 30_000,
    });
    await expect(page.getByTestId("account-wipe-error")).toContainText(
      /Current password is incorrect/i,
    );
    await expect(page.getByTestId("account-settings-container")).toBeVisible();
  });

  test("pw-account-wipe-happy", async ({ page, auth }) => {
    const email = `wipe-ok-${Date.now()}@example.com`;
    const password = "CorrectHorseBattery1!";
    await gotoAccountSettingsSignedIn(page, auth, email, password);

    await fillWipeForm(page, "DELETE", password);
    await clickTestId(page, "account-wipe-submit");

    // Redirect after logout (sanitized referer → auth redirect / sign-in).
    await expect(page).not.toHaveURL(/\/user\/account-settings/, {
      timeout: 60_000,
    });

    // Old credentials must not sign in again.
    await page.goto("/auth/signin");
    await expect(page.getByTestId("signin-container")).toBeVisible({
      timeout: 60_000,
    });
    const root = page.getByTestId("auth-dialog-root");
    await expect(root).toBeVisible({ timeout: 60_000 });
    await root
      .getByTestId("signin-email")
      .locator('input[name="email"]')
      .fill(email);
    await root
      .getByTestId("signin-password")
      .locator('input[name="password"]')
      .fill(password);
    await root.getByTestId("signin-submit").getByRole("button").click();
    await expect(root.getByTestId("signin-error")).toBeVisible({
      timeout: 30_000,
    });
  });
});
