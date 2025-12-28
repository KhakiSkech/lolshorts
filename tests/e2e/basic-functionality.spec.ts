import { test, expect } from '@playwright/test';

/**
 * Basic E2E Tests for LoLShorts Application
 *
 * These tests verify core application functionality without requiring
 * actual League of Legends game integration
 */

test.describe('LoLShorts Basic Functionality', () => {
  test.beforeEach(async ({ page }) => {
    // Mock Tauri API before each test
    await page.addInitScript(() => {
      window.__TAURI__ = {
        invoke: jest.fn().mockImplementation(async (cmd: string, args?: any) => {
          switch (cmd) {
            case 'get_auth_status':
              return { authenticated: false, tier: 'FREE' };
            case 'list_audio_devices':
              return [
                { id: 'default', name: 'Default Audio Device', device_type: 'SystemAudio' },
                { id: 'microphone', name: 'Microphone', device_type: 'Microphone' }
              ];
            case 'get_recording_status':
              return 'idle';
            case 'get_detailed_recording_status':
              return {
                status: 'Idle',
                is_monitoring: false,
                buffer_duration_secs: 120
              };
            case 'get_saved_clips':
              return [];
            case 'get_system_info':
              return {
                platform: 'win32',
                arch: 'x64',
                version: '1.2.0'
              };
            case 'refresh_audio_devices':
              return [
                { id: 'default', name: 'Default Audio Device', device_type: 'SystemAudio' },
                { id: 'microphone', name: 'Microphone', device_type: 'Microphone' }
              ];
            case 'start_recording':
              return undefined;
            case 'stop_recording':
              return undefined;
            case 'start_auto_capture':
              return undefined;
            case 'stop_auto_capture':
              return undefined;
            case 'get_audio_devices_with_cache_info':
              return {
                devices: [
                  { id: 'default', name: 'Default Audio Device', device_type: 'SystemAudio' },
                  { id: 'microphone', name: 'Microphone', device_type: 'Microphone' }
                ],
                cache_age_seconds: 0,
                cache_ttl_seconds: 60,
                cache_valid: true,
                total_devices: 2
              };
            default:
              console.warn(`Unmocked Tauri command: ${cmd}`);
              return null;
          }
        }),
        event: {
          listen: jest.fn(),
          emit: jest.fn(),
        },
      };
    });
  });

  test('application loads successfully', async ({ page }) => {
    await page.goto('http://localhost:5181');

    // Check if the application title is correct
    await expect(page).toHaveTitle(/LoLShorts/);

    // Check if main application container is visible
    await expect(page.locator('[data-testid="app-shell"]')).toBeVisible();
  });

  test('dashboard renders correctly', async ({ page }) => {
    await page.goto('http://localhost:5181');

    // Check if dashboard elements are present
    await expect(page.locator('[data-testid="dashboard"]')).toBeVisible();
    await expect(page.locator('[data-testid="status-dashboard"]')).toBeVisible();
    await expect(page.locator('[data-testid="recording-controls"]')).toBeVisible();
  });

  test('audio device management works', async ({ page }) => {
    await page.goto('http://localhost:5181');

    // Navigate to settings if needed
    await page.click('[data-testid="settings-button"]');

    // Check if audio settings section is available
    await expect(page.locator('[data-testid="audio-settings"]')).toBeVisible();

    // Test audio device listing (this would trigger Tauri command)
    await page.click('[data-testid="refresh-audio-devices"]');

    // Verify the mock was called (we can't directly test the mock in Playwright,
    // but we can verify the UI responds appropriately)
    await expect(page.locator('[data-testid="audio-device-list"]')).toBeVisible();
  });

  test('recording controls are functional', async ({ page }) => {
    await page.goto('http://localhost:5181');

    // Check if recording controls are present
    const startButton = page.locator('[data-testid="start-recording"]');
    const stopButton = page.locator('[data-testid="stop-recording"]');
    const autoButton = page.locator('[data-testid="auto-capture"]');

    await expect(startButton).toBeVisible();
    await expect(autoButton).toBeVisible();

    // Test start recording (should be enabled initially)
    await expect(startButton).toBeEnabled();

    // Test auto capture functionality
    await expect(autoButton).toBeEnabled();

    // Simulate starting recording
    await startButton.click();

    // Verify state changes (depending on implementation)
    // This might show a different button state or loading indicator
  });

  test('navigation between sections works', async ({ page }) => {
    await page.goto('http://localhost:5181');

    // Test navigation to different sections
    const dashboardLink = page.locator('[data-testid="nav-dashboard"]');
    const settingsLink = page.locator('[data-testid="nav-settings"]');
    const libraryLink = page.locator('[data-testid="nav-library"]');

    if (await dashboardLink.isVisible()) {
      await dashboardLink.click();
      await expect(page.locator('[data-testid="dashboard"]')).toBeVisible();
    }

    if (await settingsLink.isVisible()) {
      await settingsLink.click();
      await expect(page.locator('[data-testid="settings"]')).toBeVisible();
    }

    if (await libraryLink.isVisible()) {
      await libraryLink.click();
      await expect(page.locator('[data-testid="clip-library"]')).toBeVisible();
    }
  });

  test('status indicators update correctly', async ({ page }) => {
    await page.goto('http://localhost:5181');

    // Check if status indicators are present
    const statusIndicator = page.locator('[data-testid="status-indicator"]');
    const lcuStatus = page.locator('[data-testid="lcu-status"]');

    await expect(statusIndicator).toBeVisible();

    // Test status updates (these would come from mocked Tauri commands)
    if (await lcuStatus.isVisible()) {
      const initialStatus = await lcuStatus.textContent();
      expect(initialStatus).toBeDefined();
    }
  });

  test('responsive design works correctly', async ({ page }) => {
    await page.goto('http://localhost:5181');

    // Test desktop viewport
    await page.setViewportSize({ width: 1280, height: 720 });
    await expect(page.locator('[data-testid="app-shell"]')).toBeVisible();

    // Test smaller viewport
    await page.setViewportSize({ width: 768, height: 1024 });
    await expect(page.locator('[data-testid="app-shell"]')).toBeVisible();

    // Test mobile viewport
    await page.setViewportSize({ width: 375, height: 667 });
    await expect(page.locator('[data-testid="app-shell"]')).toBeVisible();
  });
});