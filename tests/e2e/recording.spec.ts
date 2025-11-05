import { test, expect } from '@playwright/test';

/**
 * E2E Tests for Recording System
 *
 * Tests:
 * - Recording start/stop
 * - Replay buffer management
 * - LCU connection status
 * - Clip capture
 * - Event detection
 * - Screenshot capture
 */

test.describe('Recording System', () => {
  test.beforeEach(async ({ page }) => {
    // Login first
    await page.goto('/');
    await page.getByRole('button', { name: /login/i }).click();
    await page.fill('input[type="email"]', 'test@lolshorts.com');
    await page.fill('input[type="password"]', 'TestPassword123!');
    await page.getByRole('button', { name: /submit|login/i }).click();
    await expect(page).toHaveURL(/\/dashboard/);

    // Navigate to recording page
    await page.goto('/recording');
  });

  test('should display recording status', async ({ page }) => {
    // Should show LCU connection status
    await expect(page.getByText(/lcu.*status|league.*client/i)).toBeVisible();

    // Should show recording controls
    await expect(page.getByRole('button', { name: /start.*record/i })).toBeVisible();
  });

  test('should show LCU disconnected state initially', async ({ page }) => {
    // Without League Client running, should show disconnected
    const lcuStatus = page.locator('[data-testid="lcu-status"]');

    // Wait for status to load
    await page.waitForTimeout(2000);

    // Should show disconnected or not running
    const statusText = await lcuStatus.textContent();
    expect(statusText).toMatch(/disconnected|not.*running|offline/i);
  });

  test('should start replay buffer when recording', async ({ page }) => {
    // Click start recording
    await page.getByRole('button', { name: /start.*record/i }).click();

    // Should show recording indicator
    await expect(page.getByText(/recording|active/i)).toBeVisible();

    // Button should change to "Stop Recording"
    await expect(page.getByRole('button', { name: /stop.*record/i })).toBeVisible();
  });

  test('should stop replay buffer', async ({ page }) => {
    // Start recording first
    await page.getByRole('button', { name: /start.*record/i }).click();
    await expect(page.getByRole('button', { name: /stop.*record/i })).toBeVisible();

    // Stop recording
    await page.getByRole('button', { name: /stop.*record/i }).click();

    // Should show stopped state
    await expect(page.getByRole('button', { name: /start.*record/i })).toBeVisible();
  });

  test('should display recent clips', async ({ page }) => {
    // Navigate to clips view
    await page.goto('/clips');

    // Should show clips list or empty state
    const hasClipsList = await page.locator('[data-testid="clips-list"]').isVisible();
    const hasEmptyState = await page.locator('text=/no clips|empty|start recording/i').isVisible();

    expect(hasClipsList || hasEmptyState).toBeTruthy();
  });

  test('should filter clips by priority', async ({ page }) => {
    // Navigate to clips view
    await page.goto('/clips');

    // Should have priority filter options
    const filterButton = page.getByRole('button', { name: /filter|priority/i });

    if (await filterButton.isVisible()) {
      await filterButton.click();

      // Should show priority options
      await expect(page.getByText(/pentakill|⭐.*5/i)).toBeVisible();
      await expect(page.getByText(/quadra|⭐.*4/i)).toBeVisible();
    }
  });

  test('should capture screenshot', async ({ page }) => {
    // Should have screenshot button
    const screenshotButton = page.getByRole('button', { name: /screenshot|capture/i });

    if (await screenshotButton.isVisible()) {
      await screenshotButton.click();

      // Should show success message
      await expect(page.getByText(/screenshot.*saved|captured/i)).toBeVisible({
        timeout: 5000,
      });
    }
  });

  test('should save manual clip', async ({ page }) => {
    // Start recording
    await page.getByRole('button', { name: /start.*record/i }).click();
    await page.waitForTimeout(2000);

    // Save clip button should be available
    const saveClipButton = page.getByRole('button', { name: /save.*clip|capture.*moment/i });

    if (await saveClipButton.isVisible()) {
      await saveClipButton.click();

      // Should show success notification
      await expect(page.getByText(/clip.*saved|saved.*successfully/i)).toBeVisible({
        timeout: 5000,
      });
    }
  });
});

test.describe('Event Detection', () => {
  test.beforeEach(async ({ page }) => {
    // Login
    await page.goto('/');
    await page.getByRole('button', { name: /login/i }).click();
    await page.fill('input[type="email"]', 'test@lolshorts.com');
    await page.fill('input[type="password"]', 'TestPassword123!');
    await page.getByRole('button', { name: /submit|login/i }).click();
  });

  test('should display detected events', async ({ page }) => {
    // Navigate to events monitor
    await page.goto('/events');

    // Should show events feed or empty state
    const hasEventsFeed = await page.locator('[data-testid="events-feed"]').isVisible();
    const hasEmptyState = await page.locator('text=/no events|waiting.*game/i').isVisible();

    expect(hasEventsFeed || hasEmptyState).toBeTruthy();
  });

  test('should show event priority badges', async ({ page }) => {
    await page.goto('/clips');

    // If clips exist, should show priority badges
    const clipCards = page.locator('[data-testid="clip-card"]');
    const clipCount = await clipCards.count();

    if (clipCount > 0) {
      // First clip should have priority badge
      const firstClip = clipCards.first();
      await expect(firstClip.locator('text=/⭐|priority/i')).toBeVisible();
    }
  });

  test('should display event types', async ({ page }) => {
    await page.goto('/clips');

    // Should show event type labels
    const eventTypes = [
      /champion.*kill/i,
      /multi.*kill/i,
      /penta.*kill/i,
      /dragon/i,
      /baron/i,
      /objective/i,
    ];

    // Check if any event types are visible
    let hasEventType = false;
    for (const eventType of eventTypes) {
      if (await page.locator(`text=${eventType}`).isVisible()) {
        hasEventType = true;
        break;
      }
    }

    // Either has event types or shows empty state
    const hasEmptyState = await page.locator('text=/no clips|empty/i').isVisible();
    expect(hasEventType || hasEmptyState).toBeTruthy();
  });
});

test.describe('Clip Management', () => {
  test.beforeEach(async ({ page }) => {
    // Login
    await page.goto('/');
    await page.getByRole('button', { name: /login/i }).click();
    await page.fill('input[type="email"]', 'test@lolshorts.com');
    await page.fill('input[type="password"]', 'TestPassword123!');
    await page.getByRole('button', { name: /submit|login/i }).click();

    await page.goto('/clips');
  });

  test('should play clip preview', async ({ page }) => {
    // Find first clip card
    const clipCard = page.locator('[data-testid="clip-card"]').first();

    if (await clipCard.isVisible()) {
      // Click play button
      await clipCard.getByRole('button', { name: /play/i }).click();

      // Should show video player
      await expect(page.locator('video')).toBeVisible();
    }
  });

  test('should delete clip', async ({ page }) => {
    const clipCard = page.locator('[data-testid="clip-card"]').first();

    if (await clipCard.isVisible()) {
      // Click delete button
      await clipCard.getByRole('button', { name: /delete|remove/i }).click();

      // Should show confirmation dialog
      await expect(page.getByText(/confirm|are you sure/i)).toBeVisible();

      // Confirm deletion
      await page.getByRole('button', { name: /confirm|yes|delete/i }).click();

      // Should show success message
      await expect(page.getByText(/deleted|removed/i)).toBeVisible();
    }
  });

  test('should export clip', async ({ page }) => {
    const clipCard = page.locator('[data-testid="clip-card"]').first();

    if (await clipCard.isVisible()) {
      // Click export button
      await clipCard.getByRole('button', { name: /export|save as/i }).click();

      // Should show export options or save dialog
      const hasExportOptions = await page.getByText(/format|quality/i).isVisible();
      expect(hasExportOptions).toBeTruthy();
    }
  });
});

test.describe('Performance', () => {
  test('should load recording page within 3 seconds', async ({ page }) => {
    await page.goto('/');
    await page.getByRole('button', { name: /login/i }).click();
    await page.fill('input[type="email"]', 'test@lolshorts.com');
    await page.fill('input[type="password"]', 'TestPassword123!');
    await page.getByRole('button', { name: /submit|login/i }).click();

    const startTime = Date.now();
    await page.goto('/recording');
    await page.waitForLoadState('networkidle');
    const loadTime = Date.now() - startTime;

    expect(loadTime).toBeLessThan(3000);
  });

  test('should handle rapid recording toggles', async ({ page }) => {
    await page.goto('/');
    await page.getByRole('button', { name: /login/i }).click();
    await page.fill('input[type="email"]', 'test@lolshorts.com');
    await page.fill('input[type="password"]', 'TestPassword123!');
    await page.getByRole('button', { name: /submit|login/i }).click();
    await page.goto('/recording');

    // Toggle recording multiple times quickly
    for (let i = 0; i < 3; i++) {
      await page.getByRole('button', { name: /start.*record/i }).click();
      await page.waitForTimeout(500);
      await page.getByRole('button', { name: /stop.*record/i }).click();
      await page.waitForTimeout(500);
    }

    // Should end in stopped state
    await expect(page.getByRole('button', { name: /start.*record/i })).toBeVisible();
  });
});
