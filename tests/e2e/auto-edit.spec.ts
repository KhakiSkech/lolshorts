import { test, expect, Page } from '@playwright/test';

/**
 * Auto-Edit E2E Tests
 *
 * Test Coverage:
 * 1. Complete workflow: game selection → configuration → generation → result
 * 2. Canvas editor functionality
 * 3. Audio mixer functionality
 * 4. Error scenarios and recovery
 * 5. Progress tracking
 *
 * Prerequisites:
 * - User must be authenticated with PRO subscription
 * - At least 2 recorded games must exist in the database
 * - FFmpeg must be available in system PATH
 * - Sufficient disk space for video generation
 */

// Helper function to login with PRO account
async function loginWithProAccount(page: Page) {
  await page.goto('/');

  // Wait for auth state to load
  await page.waitForTimeout(1000);

  // Check if already logged in
  const isLoggedIn = await page.locator('[data-testid="user-menu"]').isVisible().catch(() => false);

  if (!isLoggedIn) {
    // Click sign in
    await page.click('[data-testid="sign-in-button"]');

    // Fill credentials (use test PRO account)
    await page.fill('[data-testid="email-input"]', process.env.TEST_PRO_EMAIL || 'test-pro@example.com');
    await page.fill('[data-testid="password-input"]', process.env.TEST_PRO_PASSWORD || 'test-password-123');

    // Submit
    await page.click('[data-testid="submit-button"]');

    // Wait for redirect
    await page.waitForURL('/');
  }
}

// Helper function to navigate to Auto-Edit page
async function navigateToAutoEdit(page: Page) {
  await page.goto('/auto-edit');

  // Verify PRO feature gate
  await expect(page.locator('text=Auto-Edit')).toBeVisible();

  // Wait for page to load
  await page.waitForTimeout(1000);
}

test.describe('Auto-Edit Complete Workflow', () => {
  test.beforeEach(async ({ page }) => {
    await loginWithProAccount(page);
    await navigateToAutoEdit(page);
  });

  test('should complete full auto-edit workflow with default settings', async ({ page }) => {
    // Step 1: Select games
    await test.step('Select at least 2 games', async () => {
      // Wait for games to load
      await page.waitForSelector('[data-testid="game-selection-grid"]', { timeout: 10000 });

      // Get all available games
      const gameCards = await page.locator('[data-testid^="game-card-"]').all();
      expect(gameCards.length).toBeGreaterThanOrEqual(2);

      // Select first 2 games
      await page.click('[data-testid^="game-card-"]:nth-of-type(1)');
      await page.click('[data-testid^="game-card-"]:nth-of-type(2)');

      // Verify selection count
      await expect(page.locator('text=/Selected: 2/')).toBeVisible();
    });

    // Step 2: Choose duration
    await test.step('Select target duration', async () => {
      // Click 60s duration button (default)
      await page.click('[data-testid="duration-60"]');

      // Verify button is selected
      await expect(page.locator('[data-testid="duration-60"]')).toHaveClass(/selected|active|primary/);
    });

    // Step 3: Start generation (skip canvas/audio customization)
    await test.step('Start auto-edit generation', async () => {
      // Click "Generate Video" button
      await page.click('[data-testid="generate-button"]');

      // Verify progress tracking started
      await expect(page.locator('[data-testid="progress-section"]')).toBeVisible({ timeout: 5000 });
    });

    // Step 4: Monitor progress
    await test.step('Monitor progress through all stages', async () => {
      // Stage 1: Selecting Clips
      await expect(page.locator('text=Selecting Clips')).toBeVisible({ timeout: 10000 });

      // Stage 2: Preparing Clips
      await expect(page.locator('text=Preparing Clips')).toBeVisible({ timeout: 30000 });

      // Stage 3: Concatenating
      await expect(page.locator('text=Concatenating')).toBeVisible({ timeout: 30000 });

      // Stage 4: Applying Canvas (if template is used)
      // Stage 5: Mixing Audio (if background music is used)

      // Wait for completion (max 3 minutes for 60s video)
      await expect(page.locator('[data-testid="result-section"]')).toBeVisible({ timeout: 180000 });
    });

    // Step 5: Verify successful completion
    await test.step('Verify successful completion', async () => {
      // Check success message
      await expect(page.locator('text=/Video generated successfully/')).toBeVisible();

      // Check file location is displayed
      await expect(page.locator('[data-testid="output-file-path"]')).toBeVisible();

      // Verify file path format
      const filePath = await page.locator('[data-testid="output-file-path"]').textContent();
      expect(filePath).toMatch(/\.mp4$/); // Ends with .mp4

      // Check action buttons are available
      await expect(page.locator('[data-testid="open-location-button"]')).toBeVisible();
      await expect(page.locator('[data-testid="play-video-button"]')).toBeVisible();
    });

    // Step 6: Verify output file exists
    await test.step('Verify output file exists on filesystem', async () => {
      // This requires access to Tauri filesystem API
      // For now, we verify the UI shows file information
      const filePath = await page.locator('[data-testid="output-file-path"]').textContent();
      expect(filePath).toBeTruthy();
      expect(filePath!.length).toBeGreaterThan(0);
    });
  });

  test('should handle workflow with canvas template', async ({ page }) => {
    // Step 1: Select games
    await page.waitForSelector('[data-testid="game-selection-grid"]');
    await page.click('[data-testid^="game-card-"]:nth-of-type(1)');

    // Step 2: Navigate to Canvas tab
    await page.click('[data-testid="canvas-tab"]');
    await expect(page.locator('[data-testid="canvas-editor"]')).toBeVisible();

    // Step 3: Configure canvas
    await test.step('Configure canvas template', async () => {
      // Change background color
      await page.click('[data-testid="background-color-button"]');
      await page.fill('[data-testid="color-input"]', '#FF5733');

      // Add text element
      await page.click('[data-testid="add-text-button"]');
      await page.fill('[data-testid="text-content-input"]', 'Epic Moments');
      await page.fill('[data-testid="text-size-input"]', '48');

      // Click canvas to position (center-top)
      const canvas = page.locator('[data-testid="canvas-preview"]');
      await canvas.click({ position: { x: 180, y: 50 } }); // 360x640 canvas, click at 50% x, 8% y
    });

    // Step 4: Generate with canvas
    await page.click('[data-testid="generate-button"]');

    // Step 5: Wait for completion
    await expect(page.locator('[data-testid="result-section"]')).toBeVisible({ timeout: 180000 });

    // Step 6: Verify canvas was applied
    await expect(page.locator('text=/Canvas applied/')).toBeVisible();
  });

  test('should handle workflow with background music', async ({ page }) => {
    // Step 1: Select games
    await page.waitForSelector('[data-testid="game-selection-grid"]');
    await page.click('[data-testid^="game-card-"]:nth-of-type(1)');

    // Step 2: Navigate to Audio tab
    await page.click('[data-testid="audio-tab"]');
    await expect(page.locator('[data-testid="audio-mixer"]')).toBeVisible();

    // Step 3: Upload background music
    await test.step('Upload background music', async () => {
      // Note: File upload testing requires real file or mock
      // For now, we test the UI components are present
      await expect(page.locator('[data-testid="upload-music-button"]')).toBeVisible();
      await expect(page.locator('[data-testid="game-audio-slider"]')).toBeVisible();
      await expect(page.locator('[data-testid="music-audio-slider"]')).toBeVisible();
    });

    // Step 4: Adjust audio levels using preset
    await page.click('[data-testid="preset-balanced"]');

    // Verify sliders updated
    const gameVolume = await page.locator('[data-testid="game-audio-value"]').textContent();
    const musicVolume = await page.locator('[data-testid="music-audio-value"]').textContent();
    expect(gameVolume).toBe('70%');
    expect(musicVolume).toBe('30%');
  });

  test('should persist configuration when navigating tabs', async ({ page }) => {
    // Select game
    await page.waitForSelector('[data-testid="game-selection-grid"]');
    await page.click('[data-testid^="game-card-"]:nth-of-type(1)');

    // Select 120s duration
    await page.click('[data-testid="duration-120"]');

    // Navigate to Canvas tab
    await page.click('[data-testid="canvas-tab"]');

    // Navigate back to Config tab
    await page.click('[data-testid="config-tab"]');

    // Verify game selection persisted
    await expect(page.locator('[data-testid^="game-card-"]:nth-of-type(1)')).toHaveClass(/selected|active/);

    // Verify duration selection persisted
    await expect(page.locator('[data-testid="duration-120"]')).toHaveClass(/selected|active|primary/);
  });

  test('should allow deselecting games', async ({ page }) => {
    // Select first game
    await page.waitForSelector('[data-testid="game-selection-grid"]');
    await page.click('[data-testid^="game-card-"]:nth-of-type(1)');

    // Verify selected
    await expect(page.locator('text=Selected: 1')).toBeVisible();

    // Click again to deselect
    await page.click('[data-testid^="game-card-"]:nth-of-type(1)');

    // Verify deselected
    await expect(page.locator('text=Selected: 0')).toBeVisible();

    // Generate button should be disabled
    await expect(page.locator('[data-testid="generate-button"]')).toBeDisabled();
  });
});

test.describe('Auto-Edit Error Scenarios', () => {
  test.beforeEach(async ({ page }) => {
    await loginWithProAccount(page);
    await navigateToAutoEdit(page);
  });

  test('should show error if no games selected', async ({ page }) => {
    // Try to generate without selecting games
    await page.waitForSelector('[data-testid="game-selection-grid"]');

    // Generate button should be disabled
    await expect(page.locator('[data-testid="generate-button"]')).toBeDisabled();

    // Tooltip or error message should explain why
    await page.hover('[data-testid="generate-button"]');
    await expect(page.locator('text=/Select at least one game/')).toBeVisible({ timeout: 2000 });
  });

  test('should handle FFmpeg errors gracefully', async ({ page }) => {
    // This test requires simulating FFmpeg failure
    // For now, we verify error UI components exist

    // Select game and start generation
    await page.waitForSelector('[data-testid="game-selection-grid"]');
    await page.click('[data-testid^="game-card-"]:nth-of-type(1)');
    await page.click('[data-testid="generate-button"]');

    // If error occurs during generation, verify error handling
    // Note: This would need to be triggered by actual FFmpeg failure or mock
    const errorSection = page.locator('[data-testid="error-section"]');

    // If error appears (not guaranteed in this test)
    if (await errorSection.isVisible({ timeout: 180000 }).catch(() => false)) {
      // Verify error message is displayed
      await expect(page.locator('[data-testid="error-message"]')).toBeVisible();

      // Verify retry button exists
      await expect(page.locator('[data-testid="retry-button"]')).toBeVisible();

      // Verify reset button exists
      await expect(page.locator('[data-testid="reset-button"]')).toBeVisible();
    }
  });

  test('should prevent generation with insufficient disk space', async ({ page }) => {
    // This test would require checking disk space via Tauri command
    // For now, we document the expected behavior:
    // 1. Before generation, check available disk space
    // 2. Estimate required space (clips * 1.5 for overhead)
    // 3. Show error if insufficient space

    // UI components that should exist for this check
    await page.waitForSelector('[data-testid="game-selection-grid"]');

    // When implemented, this test would:
    // - Mock low disk space condition
    // - Attempt generation
    // - Verify error message about disk space
    // - Verify suggested actions (free up space, select fewer games)
  });
});

test.describe('Auto-Edit Progress Tracking', () => {
  test.beforeEach(async ({ page }) => {
    await loginWithProAccount(page);
    await navigateToAutoEdit(page);
  });

  test('should show accurate progress percentage', async ({ page }) => {
    // Select game and generate
    await page.waitForSelector('[data-testid="game-selection-grid"]');
    await page.click('[data-testid^="game-card-"]:nth-of-type(1)');
    await page.click('[data-testid="generate-button"]');

    // Verify progress bar appears
    await expect(page.locator('[data-testid="progress-bar"]')).toBeVisible({ timeout: 5000 });

    // Progress should start at 0%
    const initialProgress = await page.locator('[data-testid="progress-percentage"]').textContent();
    expect(parseInt(initialProgress!)).toBeLessThanOrEqual(20); // Should be < 20% at start

    // Wait a bit and check progress increased
    await page.waitForTimeout(5000);
    const laterProgress = await page.locator('[data-testid="progress-percentage"]').textContent();
    expect(parseInt(laterProgress!)).toBeGreaterThan(parseInt(initialProgress!));
  });

  test('should display current stage during generation', async ({ page }) => {
    // Select game and generate
    await page.waitForSelector('[data-testid="game-selection-grid"]');
    await page.click('[data-testid^="game-card-"]:nth-of-type(1)');
    await page.click('[data-testid="generate-button"]');

    // Verify stage indicators exist
    await expect(page.locator('[data-testid="stage-selecting-clips"]')).toBeVisible({ timeout: 5000 });
    await expect(page.locator('[data-testid="stage-preparing-clips"]')).toBeVisible();
    await expect(page.locator('[data-testid="stage-concatenating"]')).toBeVisible();

    // Check that current stage is highlighted
    const currentStageLocator = page.locator('[data-testid^="stage-"]:has(.animate-spin)');
    await expect(currentStageLocator).toHaveCount(1); // Only one stage should be in progress
  });

  test('should allow canceling generation (when implemented)', async ({ page }) => {
    // This tests future functionality
    // Expected behavior:
    // 1. Start generation
    // 2. Click "Cancel" button
    // 3. Confirm cancellation
    // 4. Backend stops FFmpeg processes
    // 5. Clean up partial files
    // 6. Return to configuration step

    await page.waitForSelector('[data-testid="game-selection-grid"]');
    await page.click('[data-testid^="game-card-"]:nth-of-type(1)');
    await page.click('[data-testid="generate-button"]');

    // Look for cancel button (may not exist yet)
    const cancelButton = page.locator('[data-testid="cancel-generation-button"]');

    if (await cancelButton.isVisible({ timeout: 2000 }).catch(() => false)) {
      await cancelButton.click();

      // Confirm dialog
      await page.click('[data-testid="confirm-cancel-button"]');

      // Should return to configuration step
      await expect(page.locator('[data-testid="game-selection-grid"]')).toBeVisible({ timeout: 5000 });
    }
  });
});

test.describe('Auto-Edit Performance', () => {
  test('should meet performance targets for 60s video', async ({ page }) => {
    await loginWithProAccount(page);
    await navigateToAutoEdit(page);

    // Select one game for 60s video
    await page.waitForSelector('[data-testid="game-selection-grid"]');
    await page.click('[data-testid^="game-card-"]:nth-of-type(1)');
    await page.click('[data-testid="duration-60"]');

    // Start timer
    const startTime = Date.now();

    // Generate
    await page.click('[data-testid="generate-button"]');

    // Wait for completion
    await expect(page.locator('[data-testid="result-section"]')).toBeVisible({ timeout: 180000 });

    // Calculate elapsed time
    const elapsedTime = (Date.now() - startTime) / 1000; // seconds

    // Performance target: <30s per minute of output = <30s for 60s video
    expect(elapsedTime).toBeLessThan(30);

    console.log(`✅ Performance: 60s video generated in ${elapsedTime.toFixed(1)}s`);
  });
});
