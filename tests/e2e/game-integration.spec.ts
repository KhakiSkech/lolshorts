import { test, expect } from '@playwright/test';
import path from 'path';
import fs from 'fs';

/**
 * E2E Tests for Game Integration - Real League of Legends Testing
 *
 * These tests require:
 * 1. League of Legends client running
 * 2. In-game access for testing
 * 3. Sufficient hardware for game recording
 *
 * Test Coverage:
 * - Live Client API connection
 * - Real game event detection
 * - Actual replay buffer recording
 * - Performance with real gameplay
 * - Clip capture from real events
 */

test.describe('Game Integration Tests', () => {
  test.skip(!process.env.RUN_INTEGRATION_TESTS, 'Set RUN_INTEGRATION_TESTS=true to run game integration tests')(
    'Game Integration - Requires League of Legends'
  );

  test.beforeEach(async ({ page }) => {
    // Clear any existing session data
    await page.context.clearCookies();

    // Login with test credentials
    await page.goto('/');
    await page.getByRole('button', { name: /login/i }).click();
    await page.fill('input[type="email"]', 'test@lolshorts.com');
    await page.fill('input[type="password"]', 'TestPassword123!');
    await page.getByRole('button', { name: /submit|login/i }).click();

    // Wait for dashboard to load
    await expect(page).toHaveURL(/\/dashboard/);
  });

  test('should connect to League Client when game is running', async ({ page }) => {
    await page.goto('/recording');

    // Wait for LCU connection status
    await page.waitForTimeout(5000);

    // Should show connected status if League Client is running
    const lcuStatus = page.locator('[data-testid="lcu-status"]');
    const statusText = await lcuStatus.textContent();

    // Check if connected (League Client running)
    const isConnected = /connected|online|running/i.test(statusText);
    const isSearching = /searching|detecting/i.test(statusText);

    expect(isConnected || isSearching).toBeTruthy();

    // Should show Live Client connection status
    const liveClientStatus = page.locator('[data-testid="live-client-status"]');
    const liveStatusText = await liveClientStatus.textContent();
    expect(liveStatusText).toMatch(/connected|disconnected|searching/i);
  });

  test('should detect game state when in champion select', async ({ page }) => {
    await page.goto('/recording');

    // Wait for game state detection
    await page.waitForTimeout(8000);

    const gameStatus = page.locator('[data-testid="game-status"]');
    const statusText = await gameStatus.textContent();

    // Should show game state when in champion select or game
    const gameStates = [
      /champion.*select/i,
      /in.*game/i,
      /loading/i,
      /spectating/i
    ];

    const hasGameState = gameStates.some(state => state.test(statusText));
    expect(hasGameState).toBeTruthy();
  });

  test('should start replay buffer with game detected', async ({ page }) => {
    await page.goto('/recording');

    // Wait for game detection
    await page.waitForTimeout(8000);

    const gameStatus = page.locator('[data-testid="game-status"]');
    const statusText = await gameStatus.textContent();

    // Only proceed if game is detected
    if (/champion.*select|in.*game/i.test(statusText)) {
      // Start recording
      const startButton = page.getByRole('button', { name: /start.*record/i });
      await startButton.click();

      // Should show recording active state
      await expect(page.locator('[data-testid="recording-status"]')).toContainText(/recording|active/i);

      // Should show replay buffer indicator
      await expect(page.locator('[data-testid="replay-buffer"]')).toBeVisible();

      // Should change button to stop recording
      await expect(page.getByRole('button', { name: /stop.*record/i })).toBeVisible();
    } else {
      test.skip(true, 'Game not detected - please start League of Legends and enter champion select');
    }
  });

  test('should capture game events from Live Client API', async ({ page }) => {
    await page.goto('/events');

    // Wait for Live Client connection
    await page.waitForTimeout(10000);

    // Should show events feed
    const eventsFeed = page.locator('[data-testid="events-feed"]');
    const hasEvents = await eventsFeed.count() > 0;

    if (hasEvents) {
      // Should show real-time events
      const firstEvent = eventsFeed.first();
      await expect(firstEvent).toBeVisible();

      // Should have event metadata
      const eventTypes = [
        /champion.*kill/i,
        /assist/i,
        /death/i,
        /level.*up/i,
        /objective/i
      ];

      let hasValidEvent = false;
      for (const eventType of eventTypes) {
        const events = page.locator(`text=${eventType}`);
        if (await events.count() > 0) {
          hasValidEvent = true;
          break;
        }
      }
      expect(hasValidEvent).toBeTruthy();
    } else {
      // Should show waiting state
      expect(page.locator('text=/waiting.*game|no.*events/i')).toBeVisible();
    }
  });

  test('should capture automatic clips based on priority events', async ({ page }) => {
    await page.goto('/recording');

    // Start recording if not already recording
    const recordingStatus = page.locator('[data-testid="recording-status"]');
    const isRecording = await recordingStatus.isVisible();

    if (!isRecording || (await recordingStatus.textContent()).includes(/idle|stopped/i)) {
      await page.getByRole('button', { name: /start.*record/i }).click();
      await page.waitForTimeout(3000);
    }

    // Wait for events to be captured
    await page.waitForTimeout(15000);

    // Check if any clips were generated
    await page.goto('/clips');

    const clipsList = page.locator('[data-testid="clips-list"]');
    const hasClips = await clipsList.count() > 0;

    if (hasClips) {
      // Should show automatic clip generation
      const firstClip = clipsList.first();
      await expect(firstClip).toBeVisible();

      // Should show priority indicators
      const priorityBadge = firstClip.locator('[data-testid="priority-badge"]');
      expect(priorityBadge).toBeVisible();

      // Should show event type
      const eventType = firstClip.locator('[data-testid="event-type"]');
      expect(eventType).toBeVisible();
    } else {
      // Should indicate no clips yet or game activity needed
      expect(page.locator('text=/no.*clips|waiting.*events|start.*game/i')).toBeVisible();
    }
  });

  test('should handle real-time performance during gameplay', async ({ page }) => {
    await page.goto('/recording');

    // Start performance monitoring
    const startTime = Date.now();

    // Start recording
    await page.getByRole('button', { name: /start.*record/i }).click();

    // Monitor performance for 30 seconds
    await page.waitForTimeout(30000);

    const recordingTime = Date.now() - startTime;

    // Should still be recording without crashes
    const recordingStatus = page.locator('[data-testid="recording-status"]');
    expect(recordingStatus).toContainText(/recording|active/i);

    // Check performance metrics
    const perfMetrics = page.locator('[data-testid="performance-metrics"]');
    if (await perfMetrics.isVisible()) {
      // Should show FPS, CPU, memory usage
      expect(await perfMetrics.textContent()).toMatch(/fps|cpu|memory|performance/i);
    }

    // Stop recording
    await page.getByRole('button', { name: /stop.*record/i }).click();

    // Should stop cleanly
    expect(page.getByRole('button', { name: /start.*record/i })).toBeVisible();

    console.log(`Recording duration: ${recordingTime}ms`);
  });

  test('should generate thumbnails from real gameplay footage', async ({ page }) => {
    await page.goto('/clips');

    // Wait for clips to be available
    await page.waitForTimeout(20000);

    const clipCards = page.locator('[data-testid="clip-card"]');
    const clipCount = await clipCards.count();

    if (clipCount > 0) {
      const firstClip = clipCards.first();

      // Should have thumbnail
      const thumbnail = firstClip.locator('[data-testid="clip-thumbnail"]');
      expect(thumbnail).toBeVisible();

      // Should generate thumbnail on demand if missing
      const hasThumbnail = await thumbnail.getAttribute('src');
      if (!hasThumbnail) {
        await thumbnail.click();

        // Should show thumbnail generation progress
        await expect(page.locator('text=/generating.*thumbnail/i')).toBeVisible({
          timeout: 10000
        });

        // Should complete and show thumbnail
        await expect(thumbnail).toHaveAttribute('src');
      }
    } else {
      test.skip(true, 'No clips available for thumbnail testing');
    }
  });
});

test.describe('Game Integration Stress Tests', () => {
  test.skip(!process.env.RUN_STRESS_TESTS, 'Set RUN_STRESS_TESTS=true to run stress tests')(
    'Stress Tests - Resource Intensive'
  );

  test('should handle extended recording sessions', async ({ page }) => {
    await page.goto('/');
    await page.getByRole('button', { name: /login/i }).click();
    await page.fill('input[type="email"]', 'test@lolshorts.com');
    await page.fill('input[type="password"]', 'TestPassword123!');
    await page.getByRole('button', { name: /submit|login/i }).click();

    await page.goto('/recording');

    // Extended recording test (5 minutes)
    await page.getByRole('button', { name: /start.*record/i }).click();

    // Monitor for memory leaks or performance degradation
    const checkpoints = [];
    for (let i = 0; i < 5; i++) {
      await page.waitForTimeout(60000); // 1 minute

      const memoryUsage = page.evaluate(() => {
        return (performance as any).memory?.usedJSHeapSize || 0;
      });

      checkpoints.push({ minute: i + 1, memory: memoryUsage });

      // Memory usage should not grow excessively
      if (i > 0 && checkpoints[i].memory > checkpoints[i-1].memory * 1.5) {
        console.warn(`Memory usage increased by >50% at minute ${i + 1}`);
      }
    }

    await page.getByRole('button', { name: /stop.*record/i }).click();

    console.log('Extended recording test completed');
    console.table(checkpoints);
  });

  test('should handle rapid event processing', async ({ page }) => {
    await page.goto('/');
    await page.getByRole('button', { name: /login/i }).click();
    await page.fill('input[type="email"]', 'test@lolshorts.com');
    await page.fill('input[type="password"]', 'TestPassword123!');
    await page.getByRole('button', { name: /submit|login/i }).click();

    // Navigate to events monitor
    await page.goto('/events');

    // Monitor event processing speed
    const eventCount = await page.evaluate(() => {
      window.eventCount = window.eventCount || 0;
      return window.eventCount;
    });

    // Wait for accumulated events
    await page.waitForTimeout(10000);

    const finalEventCount = await page.evaluate(() => {
      return window.eventCount || 0;
    });

    // Should process events without delays
    expect(finalEventCount).toBeGreaterThan(eventCount);

    console.log(`Events processed: ${finalEventCount}`);
  });
});

// Helper function to check if League of Legends is running
async function isLeagueClientRunning(): Promise<boolean> {
  try {
    const response = await fetch('https://127.0.0.1:2999/liveclientdata/allgamedata', {
      signal: AbortSignal.timeout(2000)
    });
    return response.ok;
  } catch {
    return false;
  }
}

// Global setup for game integration tests
test.beforeAll(async () => {
  if (process.env.RUN_INTEGRATION_TESTS) {
    console.log('🎮 Setting up game integration tests...');

    // Check if League of Legends is running
    const isRunning = await isLeagueClientRunning();
    if (!isRunning) {
      console.warn('⚠️  League of Legends client not detected. Some tests may be skipped.');
    } else {
      console.log('✅ League of Legends client detected.');
    }

    // Ensure test data directory exists
    const testDir = path.join(process.cwd(), 'test-data');
    if (!fs.existsSync(testDir)) {
      fs.mkdirSync(testDir, { recursive: true });
    }
  }
});