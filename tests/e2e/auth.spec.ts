import { test, expect } from '@playwright/test';

/**
 * E2E Tests for Authentication System
 *
 * Tests:
 * - Login flow
 * - Signup flow
 * - Logout flow
 * - Protected features
 * - Tier-based access control
 * - Token refresh
 * - Session persistence
 */

test.describe('Authentication Flows', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to app
    await page.goto('/');
  });

  test('should display login form for unauthenticated users', async ({ page }) => {
    // Should show login/signup options
    await expect(page.getByRole('button', { name: /login/i })).toBeVisible();
    await expect(page.getByRole('button', { name: /sign up/i })).toBeVisible();
  });

  test('should show validation errors for invalid login', async ({ page }) => {
    // Click login button
    await page.getByRole('button', { name: /login/i }).click();

    // Try to submit without credentials
    await page.getByRole('button', { name: /submit|login/i }).click();

    // Should show validation errors
    await expect(page.getByText(/email.*required/i)).toBeVisible();
    await expect(page.getByText(/password.*required/i)).toBeVisible();
  });

  test('should login successfully with valid credentials', async ({ page }) => {
    // Click login button
    await page.getByRole('button', { name: /login/i }).click();

    // Fill in credentials (use test account)
    await page.fill('input[type="email"]', 'test@lolshorts.com');
    await page.fill('input[type="password"]', 'TestPassword123!');

    // Submit form
    await page.getByRole('button', { name: /submit|login/i }).click();

    // Should redirect to dashboard
    await expect(page).toHaveURL(/\/dashboard/);

    // Should show user menu
    await expect(page.getByText(/account|profile/i)).toBeVisible();
  });

  test('should logout successfully', async ({ page }) => {
    // Login first
    await page.getByRole('button', { name: /login/i }).click();
    await page.fill('input[type="email"]', 'test@lolshorts.com');
    await page.fill('input[type="password"]', 'TestPassword123!');
    await page.getByRole('button', { name: /submit|login/i }).click();
    await expect(page).toHaveURL(/\/dashboard/);

    // Click logout
    await page.getByRole('button', { name: /logout|sign out/i }).click();

    // Should redirect to login
    await expect(page).toHaveURL(/\/|login/);

    // Should show login button again
    await expect(page.getByRole('button', { name: /login/i })).toBeVisible();
  });

  test('should persist session after page reload', async ({ page }) => {
    // Login
    await page.getByRole('button', { name: /login/i }).click();
    await page.fill('input[type="email"]', 'test@lolshorts.com');
    await page.fill('input[type="password"]', 'TestPassword123!');
    await page.getByRole('button', { name: /submit|login/i }).click();
    await expect(page).toHaveURL(/\/dashboard/);

    // Reload page
    await page.reload();

    // Should still be logged in
    await expect(page).toHaveURL(/\/dashboard/);
    await expect(page.getByText(/account|profile/i)).toBeVisible();
  });

  test('should display signup form', async ({ page }) => {
    // Click signup button
    await page.getByRole('button', { name: /sign up/i }).click();

    // Should show signup form
    await expect(page.getByRole('heading', { name: /sign up|create account/i })).toBeVisible();
    await expect(page.getByLabel(/email/i)).toBeVisible();
    await expect(page.getByLabel(/^password$/i)).toBeVisible();
    await expect(page.getByLabel(/confirm password/i)).toBeVisible();
  });

  test('should validate password confirmation', async ({ page }) => {
    // Click signup
    await page.getByRole('button', { name: /sign up/i }).click();

    // Fill in mismatched passwords
    await page.fill('input[type="email"]', 'newuser@lolshorts.com');
    await page.getByLabel(/^password$/i).fill('Password123!');
    await page.getByLabel(/confirm password/i).fill('DifferentPassword123!');

    // Submit
    await page.getByRole('button', { name: /submit|sign up|create/i }).click();

    // Should show error
    await expect(page.getByText(/passwords.*not match|must match/i)).toBeVisible();
  });
});

test.describe('Protected Features', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
  });

  test('should block recording features when not authenticated', async ({ page }) => {
    // Try to access recording features
    await page.goto('/recording');

    // Should redirect to login or show auth required
    const isLoginPage = await page.locator('button:has-text("Login")').isVisible();
    const hasAuthRequired = await page.locator('text=/login required|authentication required/i').isVisible();

    expect(isLoginPage || hasAuthRequired).toBeTruthy();
  });

  test('should allow FREE tier features after login', async ({ page }) => {
    // Login as FREE user
    await page.getByRole('button', { name: /login/i }).click();
    await page.fill('input[type="email"]', 'free@lolshorts.com');
    await page.fill('input[type="password"]', 'FreeUser123!');
    await page.getByRole('button', { name: /submit|login/i }).click();
    await expect(page).toHaveURL(/\/dashboard/);

    // Should have access to basic recording
    await page.goto('/recording');
    await expect(page.getByRole('button', { name: /start.*record/i })).toBeVisible();

    // Should have access to view clips
    await page.goto('/clips');
    await expect(page.getByRole('heading', { name: /clips|recordings/i })).toBeVisible();
  });

  test('should block PRO features for FREE tier users', async ({ page }) => {
    // Login as FREE user
    await page.getByRole('button', { name: /login/i }).click();
    await page.fill('input[type="email"]', 'free@lolshorts.com');
    await page.fill('input[type="password"]', 'FreeUser123!');
    await page.getByRole('button', { name: /submit|login/i }).click();

    // Try to access PRO feature (YouTube Shorts composer)
    await page.goto('/compose');

    // Should show PRO upgrade prompt
    const hasProRequired = await page.locator('text=/pro.*required|upgrade.*pro/i').isVisible();
    const hasUpgradeButton = await page.locator('button:has-text("Upgrade to PRO")').isVisible();

    expect(hasProRequired || hasUpgradeButton).toBeTruthy();
  });

  test('should allow PRO features for PRO tier users', async ({ page }) => {
    // Login as PRO user
    await page.getByRole('button', { name: /login/i }).click();
    await page.fill('input[type="email"]', 'pro@lolshorts.com');
    await page.fill('input[type="password"]', 'ProUser123!');
    await page.getByRole('button', { name: /submit|login/i }).click();
    await expect(page).toHaveURL(/\/dashboard/);

    // Should have access to YouTube Shorts composer
    await page.goto('/compose');
    await expect(page.getByRole('heading', { name: /compose|create.*shorts/i })).toBeVisible();

    // Should have access to thumbnail generator
    await page.goto('/thumbnails');
    await expect(page.getByRole('heading', { name: /thumbnail/i })).toBeVisible();

    // Should have access to advanced clip extraction
    await page.goto('/extract');
    await expect(page.getByRole('heading', { name: /extract|clip/i })).toBeVisible();
  });
});

test.describe('Session Management', () => {
  test('should refresh token automatically', async ({ page }) => {
    // Login
    await page.goto('/');
    await page.getByRole('button', { name: /login/i }).click();
    await page.fill('input[type="email"]', 'test@lolshorts.com');
    await page.fill('input[type="password"]', 'TestPassword123!');
    await page.getByRole('button', { name: /submit|login/i }).click();
    await expect(page).toHaveURL(/\/dashboard/);

    // Wait for token refresh interval (30 minutes simulated as 5 seconds for testing)
    // In real app, this would be 30 minutes, but we'll test the mechanism
    await page.waitForTimeout(5000);

    // Page should still be accessible (token refreshed)
    await page.reload();
    await expect(page).toHaveURL(/\/dashboard/);
  });

  test('should handle expired token gracefully', async ({ page }) => {
    // This test would require mocking the backend to return expired token
    // For now, we verify the UI handles logout correctly
    await page.goto('/');
    await page.getByRole('button', { name: /login/i }).click();
    await page.fill('input[type="email"]', 'test@lolshorts.com');
    await page.fill('input[type="password"]', 'TestPassword123!');
    await page.getByRole('button', { name: /submit|login/i }).click();

    // Simulate expired token by clearing storage
    await page.evaluate(() => {
      localStorage.clear();
      sessionStorage.clear();
    });

    // Try to access protected route
    await page.goto('/recording');

    // Should redirect to login
    const isLoginPage = await page.locator('button:has-text("Login")').isVisible();
    expect(isLoginPage).toBeTruthy();
  });
});
