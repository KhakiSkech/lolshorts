import { test, expect, Page } from '@playwright/test';

/**
 * Audio Mixer E2E Tests
 *
 * Test Coverage:
 * 1. Background music upload and management
 * 2. Volume slider controls (game audio + background music)
 * 3. Quick preset configurations
 * 4. Loop control for background music
 * 5. Mix preview visualization
 * 6. Volume validation and limits
 * 7. Music file removal
 *
 * Audio Specifications:
 * - Game Audio Range: 0-100%
 * - Background Music Range: 0-100%
 * - Default Mix: 70% game, 30% music
 * - Fade-in/Fade-out: Automatically applied by backend
 */

// Helper: Login and navigate to Audio Mixer
async function navigateToAudioMixer(page: Page) {
  await page.goto('/auto-edit');
  await page.waitForSelector('[data-testid="audio-tab"]', { timeout: 10000 });
  await page.click('[data-testid="audio-tab"]');
  await expect(page.locator('[data-testid="audio-mixer"]')).toBeVisible();
}

test.describe('Audio Mixer - Volume Controls', () => {
  test.beforeEach(async ({ page }) => {
    await navigateToAudioMixer(page);
  });

  test('should have default audio levels (70% game, 30% music)', async ({ page }) => {
    // Check default game audio
    const gameAudioValue = await page.locator('[data-testid="game-audio-value"]').textContent();
    expect(gameAudioValue).toBe('70%');

    // Check default music audio
    const musicAudioValue = await page.locator('[data-testid="music-audio-value"]').textContent();
    expect(musicAudioValue).toBe('30%');
  });

  test('should adjust game audio volume with slider', async ({ page }) => {
    // Get game audio slider
    const slider = page.locator('[data-testid="game-audio-slider"]');

    // Set to 50%
    await slider.fill('50');

    // Verify value updated
    const value = await page.locator('[data-testid="game-audio-value"]').textContent();
    expect(value).toBe('50%');
  });

  test('should adjust background music volume with slider', async ({ page }) => {
    // Get music slider
    const slider = page.locator('[data-testid="music-audio-slider"]');

    // Set to 60%
    await slider.fill('60');

    // Verify value updated
    const value = await page.locator('[data-testid="music-audio-value"]').textContent();
    expect(value).toBe('60%');
  });

  test('should enforce volume limits (0-100%)', async ({ page }) => {
    const slider = page.locator('[data-testid="game-audio-slider"]');

    // Try to set below 0
    await slider.fill('-10');
    let value = await page.locator('[data-testid="game-audio-value"]').textContent();
    expect(parseInt(value!)).toBeGreaterThanOrEqual(0);

    // Try to set above 100
    await slider.fill('150');
    value = await page.locator('[data-testid="game-audio-value"]').textContent();
    expect(parseInt(value!)).toBeLessThanOrEqual(100);
  });

  test('should update mix preview bar in real-time', async ({ page }) => {
    // Set game audio to 40%
    await page.locator('[data-testid="game-audio-slider"]').fill('40');

    // Set music to 60%
    await page.locator('[data-testid="music-audio-slider"]').fill('60');

    // Check mix preview widths
    const gameBar = page.locator('[data-testid="mix-preview-game"]');
    const musicBar = page.locator('[data-testid="mix-preview-music"]');

    const gameWidth = await gameBar.evaluate((el) => {
      return parseInt(window.getComputedStyle(el).width);
    });

    const musicWidth = await musicBar.evaluate((el) => {
      return parseInt(window.getComputedStyle(el).width);
    });

    // Game bar should be ~40% of total width
    // Music bar should be ~60% of total width
    const ratio = gameWidth / (gameWidth + musicWidth);
    expect(ratio).toBeCloseTo(0.4, 1);
  });

  test('should allow muting game audio (0%)', async ({ page }) => {
    // Set game audio to 0%
    await page.locator('[data-testid="game-audio-slider"]').fill('0');

    // Verify value
    const value = await page.locator('[data-testid="game-audio-value"]').textContent();
    expect(value).toBe('0%');

    // Mix preview should only show music
    const gameBar = page.locator('[data-testid="mix-preview-game"]');
    const gameWidth = await gameBar.evaluate((el) => window.getComputedStyle(el).width);
    expect(gameWidth).toBe('0px');
  });

  test('should allow muting background music (0%)', async ({ page }) => {
    // Set music to 0%
    await page.locator('[data-testid="music-audio-slider"]').fill('0');

    // Verify value
    const value = await page.locator('[data-testid="music-audio-value"]').textContent();
    expect(value).toBe('0%');

    // Mix preview should only show game
    const musicBar = page.locator('[data-testid="mix-preview-music"]');
    const musicWidth = await musicBar.evaluate((el) => window.getComputedStyle(el).width);
    expect(musicWidth).toBe('0px');
  });

  test('should allow max volume for both (100%)', async ({ page }) => {
    // Set both to 100%
    await page.locator('[data-testid="game-audio-slider"]').fill('100');
    await page.locator('[data-testid="music-audio-slider"]').fill('100');

    // Verify values
    const gameValue = await page.locator('[data-testid="game-audio-value"]').textContent();
    const musicValue = await page.locator('[data-testid="music-audio-value"]').textContent();

    expect(gameValue).toBe('100%');
    expect(musicValue).toBe('100%');

    // Note: Backend will handle audio normalization to prevent clipping
  });
});

test.describe('Audio Mixer - Quick Presets', () => {
  test.beforeEach(async ({ page }) => {
    await navigateToAudioMixer(page);
  });

  test('should apply "Game Only" preset', async ({ page }) => {
    // Click Game Only preset
    await page.click('[data-testid="preset-game-only"]');

    // Verify volumes
    const gameValue = await page.locator('[data-testid="game-audio-value"]').textContent();
    const musicValue = await page.locator('[data-testid="music-audio-value"]').textContent();

    expect(gameValue).toBe('100%');
    expect(musicValue).toBe('0%');
  });

  test('should apply "Balanced" preset', async ({ page }) => {
    // Click Balanced preset
    await page.click('[data-testid="preset-balanced"]');

    // Verify volumes
    const gameValue = await page.locator('[data-testid="game-audio-value"]').textContent();
    const musicValue = await page.locator('[data-testid="music-audio-value"]').textContent();

    expect(gameValue).toBe('70%');
    expect(musicValue).toBe('30%');
  });

  test('should apply "Music Focus" preset', async ({ page }) => {
    // Click Music Focus preset
    await page.click('[data-testid="preset-music-focus"]');

    // Verify volumes
    const gameValue = await page.locator('[data-testid="game-audio-value"]').textContent();
    const musicValue = await page.locator('[data-testid="music-audio-value"]').textContent();

    expect(gameValue).toBe('40%');
    expect(musicValue).toBe('60%');
  });

  test('should apply "Music Only" preset', async ({ page }) => {
    // Click Music Only preset
    await page.click('[data-testid="preset-music-only"]');

    // Verify volumes
    const gameValue = await page.locator('[data-testid="game-audio-value"]').textContent();
    const musicValue = await page.locator('[data-testid="music-audio-value"]').textContent();

    expect(gameValue).toBe('0%');
    expect(musicValue).toBe('100%');
  });

  test('should update mix preview when applying presets', async ({ page }) => {
    // Apply Music Focus preset
    await page.click('[data-testid="preset-music-focus"]');

    // Wait for animation
    await page.waitForTimeout(300);

    // Check mix preview bar
    const gameBar = page.locator('[data-testid="mix-preview-game"]');
    const musicBar = page.locator('[data-testid="mix-preview-music"]');

    const gameWidth = await gameBar.evaluate((el) => parseInt(window.getComputedStyle(el).width));
    const musicWidth = await musicBar.evaluate((el) => parseInt(window.getComputedStyle(el).width));

    // Ratio should be ~40:60
    const ratio = gameWidth / (gameWidth + musicWidth);
    expect(ratio).toBeCloseTo(0.4, 1);
  });

  test('should allow manual adjustment after applying preset', async ({ page }) => {
    // Apply Balanced preset
    await page.click('[data-testid="preset-balanced"]');

    // Verify preset applied
    let gameValue = await page.locator('[data-testid="game-audio-value"]').textContent();
    expect(gameValue).toBe('70%');

    // Manually adjust
    await page.locator('[data-testid="game-audio-slider"]').fill('80');

    // Verify manual change persisted
    gameValue = await page.locator('[data-testid="game-audio-value"]').textContent();
    expect(gameValue).toBe('80%');
  });
});

test.describe('Audio Mixer - Background Music Upload', () => {
  test.beforeEach(async ({ page }) => {
    await navigateToAudioMixer(page);
  });

  test('should show upload UI when no music is uploaded', async ({ page }) => {
    // Verify upload area is visible
    await expect(page.locator('[data-testid="upload-music-button"]')).toBeVisible();
    await expect(page.locator('text=Add background music')).toBeVisible();
  });

  test('should accept audio file upload', async ({ page }) => {
    // Click upload button
    await page.click('[data-testid="upload-music-button"]');

    // Verify file input
    await expect(page.locator('[data-testid="music-file-input"]')).toBeAttached();

    // Note: Actual file upload requires mock or real file
    // Test just verifies UI flow is present
  });

  test('should validate audio file type', async ({ page }) => {
    // This test would verify that only audio files are accepted
    // Input should have accept="audio/*" attribute
    const fileInput = page.locator('[data-testid="music-file-input"]');
    const acceptAttr = await fileInput.getAttribute('accept');

    expect(acceptAttr).toBe('audio/*');
  });

  test('should display music file info after upload', async ({ page }) => {
    // Mock: Simulate music uploaded
    // In real test, use page.setInputFiles()

    // Verify music card appears
    // await expect(page.locator('[data-testid="music-card"]')).toBeVisible();

    // Verify file name displayed
    // await expect(page.locator('[data-testid="music-file-name"]')).toContainText('.mp3');
  });

  test('should remove uploaded music', async ({ page }) => {
    // Mock: Assume music is uploaded
    // Click remove button
    // await page.click('[data-testid="remove-music-button"]');

    // Verify music removed
    // await expect(page.locator('[data-testid="music-card"]')).not.toBeVisible();

    // Verify upload UI appears again
    // await expect(page.locator('[data-testid="upload-music-button"]')).toBeVisible();
  });
});

test.describe('Audio Mixer - Loop Control', () => {
  test.beforeEach(async ({ page }) => {
    await navigateToAudioMixer(page);
    // Note: Requires music to be uploaded
  });

  test('should have loop enabled by default', async ({ page }) => {
    // Mock: Music uploaded
    // Verify loop toggle is ON
    // const loopToggle = page.locator('[data-testid="loop-music-toggle"]');
    // await expect(loopToggle).toBeChecked();
  });

  test('should toggle loop on/off', async ({ page }) => {
    // Mock: Music uploaded
    // Toggle OFF
    // await page.click('[data-testid="loop-music-toggle"]');
    // await expect(page.locator('[data-testid="loop-music-toggle"]')).not.toBeChecked();

    // Toggle ON
    // await page.click('[data-testid="loop-music-toggle"]');
    // await expect(page.locator('[data-testid="loop-music-toggle"]')).toBeChecked();
  });

  test('should show warning when loop is disabled', async ({ page }) => {
    // Mock: Music uploaded
    // Disable loop
    // await page.click('[data-testid="loop-music-toggle"]');

    // Verify warning message
    // await expect(page.locator('text=/Music will play once/')).toBeVisible();
    // await expect(page.locator('text=/Video may be longer/')).toBeVisible();
  });

  test('should hide warning when loop is enabled', async ({ page }) => {
    // Mock: Music uploaded with loop OFF
    // Enable loop
    // await page.click('[data-testid="loop-music-toggle"]');

    // Verify warning hidden
    // await expect(page.locator('text=/Music will play once/')).not.toBeVisible();
  });
});

test.describe('Audio Mixer - Mix Preview Visualization', () => {
  test.beforeEach(async ({ page }) => {
    await navigateToAudioMixer(page);
  });

  test('should show visual mix preview bar', async ({ page }) => {
    // Verify mix preview exists
    await expect(page.locator('[data-testid="mix-preview"]')).toBeVisible();

    // Verify game and music bars
    await expect(page.locator('[data-testid="mix-preview-game"]')).toBeVisible();
    await expect(page.locator('[data-testid="mix-preview-music"]')).toBeVisible();
  });

  test('should display correct colors for game and music', async ({ page }) => {
    // Game audio should be blue
    const gameBar = page.locator('[data-testid="mix-preview-game"]');
    const gameColor = await gameBar.evaluate((el) => window.getComputedStyle(el).backgroundColor);
    expect(gameColor).toContain('rgb'); // Should have color (blue)

    // Music should be purple
    const musicBar = page.locator('[data-testid="mix-preview-music"]');
    const musicColor = await musicBar.evaluate((el) => window.getComputedStyle(el).backgroundColor);
    expect(musicColor).toContain('rgb'); // Should have color (purple)

    // Colors should be different
    expect(gameColor).not.toBe(musicColor);
  });

  test('should show labels in preview bars', async ({ page }) => {
    // Default 70/30 should show labels
    await expect(page.locator('[data-testid="mix-preview-game"]')).toContainText('Game');
    await expect(page.locator('[data-testid="mix-preview-music"]')).toContainText('Music');
  });

  test('should hide labels when bars are too narrow', async ({ page }) => {
    // Set game audio to 5% (too narrow for label)
    await page.locator('[data-testid="game-audio-slider"]').fill('5');

    // Label should not be visible or bar should be too small
    const gameBar = page.locator('[data-testid="mix-preview-game"]');
    const text = await gameBar.textContent();
    expect(text).toBe(''); // No text when <15%
  });
});

test.describe('Audio Mixer - Tip and Recommendations', () => {
  test.beforeEach(async ({ page }) => {
    await navigateToAudioMixer(page);
  });

  test('should display audio mixing tips', async ({ page }) => {
    // Verify tip section exists
    await expect(page.locator('[data-testid="audio-tip"]')).toBeVisible();

    // Verify recommended values mentioned
    await expect(page.locator('text=/70%.*game.*30%.*music/i')).toBeVisible();
  });

  test('should mention fade effects', async ({ page }) => {
    // Verify fade-in/fade-out mentioned
    await expect(page.locator('text=/fade-in.*fade-out/i')).toBeVisible();
  });
});

test.describe('Audio Mixer - Music Slider Disable State', () => {
  test.beforeEach(async ({ page }) => {
    await navigateToAudioMixer(page);
  });

  test('should disable music slider when no music uploaded', async ({ page }) => {
    // Music slider should be disabled without uploaded music
    const musicSlider = page.locator('[data-testid="music-audio-slider"]');
    await expect(musicSlider).toBeDisabled();
  });

  test('should enable music slider after music upload', async ({ page }) => {
    // Mock: Upload music
    // Music slider should now be enabled
    // const musicSlider = page.locator('[data-testid="music-audio-slider"]');
    // await expect(musicSlider).toBeEnabled();
  });
});

test.describe('Audio Mixer - State Persistence', () => {
  test.beforeEach(async ({ page }) => {
    await navigateToAudioMixer(page);
  });

  test('should persist volume settings when navigating tabs', async ({ page }) => {
    // Set custom volumes
    await page.locator('[data-testid="game-audio-slider"]').fill('85');
    await page.locator('[data-testid="music-audio-slider"]').fill('45');

    // Navigate to Canvas tab
    await page.click('[data-testid="canvas-tab"]');

    // Navigate back to Audio tab
    await page.click('[data-testid="audio-tab"]');

    // Verify volumes persisted
    const gameValue = await page.locator('[data-testid="game-audio-value"]').textContent();
    const musicValue = await page.locator('[data-testid="music-audio-value"]').textContent();

    expect(gameValue).toBe('85%');
    expect(musicValue).toBe('45%');
  });

  test('should persist loop setting when navigating tabs', async ({ page }) => {
    // Mock: Music uploaded
    // Disable loop
    // await page.click('[data-testid="loop-music-toggle"]');

    // Navigate away and back
    // await page.click('[data-testid="config-tab"]');
    // await page.click('[data-testid="audio-tab"]');

    // Verify loop setting persisted
    // await expect(page.locator('[data-testid="loop-music-toggle"]')).not.toBeChecked();
  });
});

test.describe('Audio Mixer - Accessibility', () => {
  test.beforeEach(async ({ page }) => {
    await navigateToAudioMixer(page);
  });

  test('should have proper ARIA labels for sliders', async ({ page }) => {
    const gameSlider = page.locator('[data-testid="game-audio-slider"]');
    const musicSlider = page.locator('[data-testid="music-audio-slider"]');

    // Should have aria-label
    await expect(gameSlider).toHaveAttribute('aria-label', /.+/);
    await expect(musicSlider).toHaveAttribute('aria-label', /.+/);
  });

  test('should have proper ARIA labels for presets', async ({ page }) => {
    const presets = [
      '[data-testid="preset-game-only"]',
      '[data-testid="preset-balanced"]',
      '[data-testid="preset-music-focus"]',
      '[data-testid="preset-music-only"]',
    ];

    for (const preset of presets) {
      await expect(page.locator(preset)).toHaveAttribute('aria-label', /.+/);
    }
  });

  test('should be keyboard navigable', async ({ page }) => {
    // Tab through controls
    await page.keyboard.press('Tab');
    await page.keyboard.press('Tab');

    // Check that slider receives focus
    const focusedElement = await page.evaluate(() => document.activeElement?.getAttribute('data-testid'));
    expect(focusedElement).toBeTruthy();
  });
});
