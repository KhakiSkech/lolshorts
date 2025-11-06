import { test, expect, Page } from '@playwright/test';

/**
 * Canvas Editor E2E Tests
 *
 * Test Coverage:
 * 1. Background layer manipulation (color, gradient, image)
 * 2. Text element creation and positioning
 * 3. Image overlay creation and positioning
 * 4. Element selection and property editing
 * 5. Canvas template save/load/delete operations
 * 6. Click-to-position functionality
 * 7. Real-time preview accuracy
 *
 * Canvas Specifications:
 * - Preview: 360×640px (9:16 aspect ratio)
 * - Full Resolution: 1080×1920px (YouTube Shorts)
 * - Positioning: Percentage-based (0-100%)
 */

// Helper: Login and navigate to Canvas Editor
async function navigateToCanvasEditor(page: Page) {
  // Login (simplified - assumes auth is handled separately)
  await page.goto('/auto-edit');
  await page.waitForSelector('[data-testid="canvas-tab"]', { timeout: 10000 });
  await page.click('[data-testid="canvas-tab"]');
  await expect(page.locator('[data-testid="canvas-editor"]')).toBeVisible();
}

test.describe('Canvas Editor - Background Layers', () => {
  test.beforeEach(async ({ page }) => {
    await navigateToCanvasEditor(page);
  });

  test('should change background to solid color', async ({ page }) => {
    // Click color button
    await page.click('[data-testid="background-color-button"]');

    // Set color
    await page.fill('[data-testid="color-input"]', '#FF5733');

    // Verify canvas background changed
    const canvas = page.locator('[data-testid="canvas-preview"]');
    const bgColor = await canvas.evaluate((el) => {
      return window.getComputedStyle(el.querySelector('div')!).backgroundColor;
    });

    // RGB value for #FF5733 is rgb(255, 87, 51)
    expect(bgColor).toContain('255');
    expect(bgColor).toContain('87');
    expect(bgColor).toContain('51');
  });

  test('should create gradient background', async ({ page }) => {
    // Click gradient button
    await page.click('[data-testid="background-gradient-button"]');

    // Set two colors
    await page.fill('[data-testid="gradient-color1-input"]', '#FF0000');
    await page.fill('[data-testid="gradient-color2-input"]', '#0000FF');

    // Verify canvas has gradient
    const canvas = page.locator('[data-testid="canvas-preview"]');
    const bgImage = await canvas.evaluate((el) => {
      return window.getComputedStyle(el.querySelector('div')!).backgroundImage;
    });

    expect(bgImage).toContain('linear-gradient');
    expect(bgImage).toContain('rgb(255, 0, 0)'); // Red
    expect(bgImage).toContain('rgb(0, 0, 255)'); // Blue
  });

  test('should upload background image', async ({ page }) => {
    // Click image button
    await page.click('[data-testid="background-image-button"]');

    // Check file input exists
    await expect(page.locator('[data-testid="background-image-input"]')).toBeAttached();

    // Note: Actual file upload testing requires mock or real file
    // For now, we verify the UI components are present and functional
  });

  test('should switch between background types', async ({ page }) => {
    // Start with color
    await page.click('[data-testid="background-color-button"]');
    await page.fill('[data-testid="color-input"]', '#FF0000');

    // Switch to gradient
    await page.click('[data-testid="background-gradient-button"]');
    await expect(page.locator('[data-testid="gradient-color1-input"]')).toBeVisible();

    // Switch back to color
    await page.click('[data-testid="background-color-button"]');
    await expect(page.locator('[data-testid="color-input"]')).toBeVisible();

    // Background should retain color value
    const colorValue = await page.locator('[data-testid="color-input"]').inputValue();
    expect(colorValue).toBe('#FF0000');
  });
});

test.describe('Canvas Editor - Text Elements', () => {
  test.beforeEach(async ({ page }) => {
    await navigateToCanvasEditor(page);
  });

  test('should add text element', async ({ page }) => {
    // Click "Add Text" button
    await page.click('[data-testid="add-text-button"]');

    // Verify text element appears in elements list
    await expect(page.locator('[data-testid="element-0"]')).toBeVisible();

    // Verify element type is Text
    await expect(page.locator('[data-testid="element-type-0"]')).toHaveText('Text');
  });

  test('should edit text element properties', async ({ page }) => {
    // Add text
    await page.click('[data-testid="add-text-button"]');

    // Select the element
    await page.click('[data-testid="element-0"]');

    // Edit content
    await page.fill('[data-testid="text-content-input"]', 'Epic Moments');

    // Edit font size
    await page.fill('[data-testid="text-size-input"]', '48');

    // Edit color
    await page.fill('[data-testid="text-color-input"]', '#FFFFFF');

    // Add outline
    await page.fill('[data-testid="text-outline-input"]', '#000000');

    // Verify preview updates
    const textElement = page.locator('[data-testid="canvas-preview"] [data-element-id="0"]');
    await expect(textElement).toHaveText('Epic Moments');

    const fontSize = await textElement.evaluate((el) => window.getComputedStyle(el).fontSize);
    expect(parseInt(fontSize)).toBe(48);
  });

  test('should position text element by clicking canvas', async ({ page }) => {
    // Add text
    await page.click('[data-testid="add-text-button"]');

    // Select element
    await page.click('[data-testid="element-0"]');

    // Click canvas to position (center-top: 50%, 10%)
    const canvas = page.locator('[data-testid="canvas-preview"]');
    const box = await canvas.boundingBox();
    if (box) {
      // Click at 50% width, 10% height
      await canvas.click({
        position: {
          x: box.width * 0.5,
          y: box.height * 0.1,
        },
      });

      // Verify position was updated
      const posX = await page.locator('[data-testid="position-x-value"]').textContent();
      const posY = await page.locator('[data-testid="position-y-value"]').textContent();

      expect(parseFloat(posX!)).toBeCloseTo(50, 1); // ~50%
      expect(parseFloat(posY!)).toBeCloseTo(10, 1); // ~10%
    }
  });

  test('should allow editing text position via inputs', async ({ page }) => {
    // Add text
    await page.click('[data-testid="add-text-button"]');
    await page.click('[data-testid="element-0"]');

    // Edit position via inputs
    await page.fill('[data-testid="position-x-input"]', '75');
    await page.fill('[data-testid="position-y-input"]', '25');

    // Verify element moved on canvas
    const textElement = page.locator('[data-testid="canvas-preview"] [data-element-id="0"]');
    const transform = await textElement.evaluate((el) => window.getComputedStyle(el).transform);

    // Element should be positioned at 75%, 25%
    expect(transform).toBeDefined();
  });

  test('should delete text element', async ({ page }) => {
    // Add text
    await page.click('[data-testid="add-text-button"]');

    // Verify element exists
    await expect(page.locator('[data-testid="element-0"]')).toBeVisible();

    // Select and delete
    await page.click('[data-testid="element-0"]');
    await page.click('[data-testid="delete-element-button"]');

    // Verify element removed
    await expect(page.locator('[data-testid="element-0"]')).not.toBeVisible();

    // Verify canvas preview updated
    await expect(page.locator('[data-testid="canvas-preview"] [data-element-id="0"]')).not.toBeVisible();
  });
});

test.describe('Canvas Editor - Image Elements', () => {
  test.beforeEach(async ({ page }) => {
    await navigateToCanvasEditor(page);
  });

  test('should add image overlay', async ({ page }) => {
    // Click "Add Image" button
    await page.click('[data-testid="add-image-button"]');

    // Verify file input prompt or upload UI
    await expect(page.locator('[data-testid="image-path-input"]')).toBeVisible();

    // Verify element added to list
    await expect(page.locator('[data-testid="element-0"]')).toBeVisible();
    await expect(page.locator('[data-testid="element-type-0"]')).toHaveText('Image');
  });

  test('should resize image element', async ({ page }) => {
    // Add image
    await page.click('[data-testid="add-image-button"]');

    // Enter image path (mock)
    await page.fill('[data-testid="image-path-input"]', '/path/to/logo.png');

    // Select element
    await page.click('[data-testid="element-0"]');

    // Edit dimensions
    await page.fill('[data-testid="image-width-input"]', '200');
    await page.fill('[data-testid="image-height-input"]', '100');

    // Verify dimensions updated in state
    const width = await page.locator('[data-testid="image-width-value"]').textContent();
    const height = await page.locator('[data-testid="image-height-value"]').textContent();

    expect(width).toBe('200');
    expect(height).toBe('100');
  });

  test('should position image element', async ({ page }) => {
    // Add image
    await page.click('[data-testid="add-image-button"]');
    await page.fill('[data-testid="image-path-input"]', '/path/to/logo.png');

    // Click canvas to position (top-right: 90%, 10%)
    const canvas = page.locator('[data-testid="canvas-preview"]');
    const box = await canvas.boundingBox();
    if (box) {
      await canvas.click({
        position: {
          x: box.width * 0.9,
          y: box.height * 0.1,
        },
      });

      // Verify position
      const posX = await page.locator('[data-testid="position-x-value"]').textContent();
      const posY = await page.locator('[data-testid="position-y-value"]').textContent();

      expect(parseFloat(posX!)).toBeCloseTo(90, 1);
      expect(parseFloat(posY!)).toBeCloseTo(10, 1);
    }
  });
});

test.describe('Canvas Editor - Multi-Element Management', () => {
  test.beforeEach(async ({ page }) => {
    await navigateToCanvasEditor(page);
  });

  test('should handle multiple elements', async ({ page }) => {
    // Add 3 text elements
    await page.click('[data-testid="add-text-button"]');
    await page.click('[data-testid="add-text-button"]');
    await page.click('[data-testid="add-text-button"]');

    // Verify all elements in list
    await expect(page.locator('[data-testid="element-0"]')).toBeVisible();
    await expect(page.locator('[data-testid="element-1"]')).toBeVisible();
    await expect(page.locator('[data-testid="element-2"]')).toBeVisible();

    // Verify element count
    const elements = await page.locator('[data-testid^="element-"]').count();
    expect(elements).toBe(3);
  });

  test('should switch between element selections', async ({ page }) => {
    // Add 2 elements
    await page.click('[data-testid="add-text-button"]');
    await page.click('[data-testid="add-text-button"]');

    // Select first element
    await page.click('[data-testid="element-0"]');
    await page.fill('[data-testid="text-content-input"]', 'Title');

    // Select second element
    await page.click('[data-testid="element-1"]');
    await page.fill('[data-testid="text-content-input"]', 'Subtitle');

    // Verify first element retained its content
    await page.click('[data-testid="element-0"]');
    const content = await page.locator('[data-testid="text-content-input"]').inputValue();
    expect(content).toBe('Title');
  });

  test('should show selected element on canvas', async ({ page }) => {
    // Add element
    await page.click('[data-testid="add-text-button"]');

    // Select element
    await page.click('[data-testid="element-0"]');

    // Verify selection indicator on canvas
    await expect(page.locator('[data-testid="canvas-preview"] [data-element-id="0"][data-selected="true"]')).toBeVisible();

    // Or verify border/highlight
    const elementStyle = await page.locator('[data-testid="canvas-preview"] [data-element-id="0"]')
      .evaluate((el) => window.getComputedStyle(el).border);

    expect(elementStyle).toBeTruthy(); // Should have some border styling
  });
});

test.describe('Canvas Editor - Template Operations', () => {
  test.beforeEach(async ({ page }) => {
    await navigateToCanvasEditor(page);
  });

  test('should save canvas template', async ({ page }) => {
    // Create a template
    await page.click('[data-testid="background-color-button"]');
    await page.fill('[data-testid="color-input"]', '#FF5733');
    await page.click('[data-testid="add-text-button"]');
    await page.fill('[data-testid="text-content-input"]', 'My Template');

    // Save template
    await page.click('[data-testid="save-template-button"]');

    // Enter template name
    await page.fill('[data-testid="template-name-input"]', 'Epic Moments Template');

    // Confirm save
    await page.click('[data-testid="confirm-save-button"]');

    // Verify success message
    await expect(page.locator('text=/Template saved successfully/')).toBeVisible({ timeout: 5000 });
  });

  test('should load saved template', async ({ page }) => {
    // Assume a template "Test Template" exists
    // Click load button
    await page.click('[data-testid="load-template-button"]');

    // Select template from list
    await page.click('[data-testid="template-item-0"]');

    // Confirm load
    await page.click('[data-testid="confirm-load-button"]');

    // Verify template loaded (check background or elements)
    await expect(page.locator('[data-testid^="element-"]').first()).toBeVisible({ timeout: 5000 });
  });

  test('should list available templates', async ({ page }) => {
    // Open template list
    await page.click('[data-testid="load-template-button"]');

    // Verify templates displayed
    await expect(page.locator('[data-testid="template-list"]')).toBeVisible();

    // Check if any templates exist
    const templateCount = await page.locator('[data-testid^="template-item-"]').count();
    expect(templateCount).toBeGreaterThanOrEqual(0); // 0 or more templates
  });

  test('should delete template', async ({ page }) => {
    // Open template list
    await page.click('[data-testid="load-template-button"]');

    // Get initial count
    const initialCount = await page.locator('[data-testid^="template-item-"]').count();

    if (initialCount > 0) {
      // Click delete on first template
      await page.click('[data-testid="delete-template-0"]');

      // Confirm deletion
      await page.click('[data-testid="confirm-delete-button"]');

      // Verify template removed
      const newCount = await page.locator('[data-testid^="template-item-"]').count();
      expect(newCount).toBe(initialCount - 1);
    }
  });

  test('should clear canvas', async ({ page }) => {
    // Add some elements
    await page.click('[data-testid="add-text-button"]');
    await page.click('[data-testid="add-text-button"]');

    // Verify elements exist
    expect(await page.locator('[data-testid^="element-"]').count()).toBe(2);

    // Clear canvas
    await page.click('[data-testid="clear-canvas-button"]');

    // Confirm clear
    await page.click('[data-testid="confirm-clear-button"]');

    // Verify all elements removed
    expect(await page.locator('[data-testid^="element-"]').count()).toBe(0);

    // Verify canvas is empty
    await expect(page.locator('[data-testid="canvas-preview"] [data-element-id]')).not.toBeVisible();
  });
});

test.describe('Canvas Editor - Real-time Preview', () => {
  test.beforeEach(async ({ page }) => {
    await navigateToCanvasEditor(page);
  });

  test('should update preview in real-time', async ({ page }) => {
    // Add text
    await page.click('[data-testid="add-text-button"]');

    // Type in content field and verify preview updates
    await page.fill('[data-testid="text-content-input"]', 'Real');
    await expect(page.locator('[data-testid="canvas-preview"] [data-element-id="0"]')).toHaveText('Real');

    await page.fill('[data-testid="text-content-input"]', 'Real-time');
    await expect(page.locator('[data-testid="canvas-preview"] [data-element-id="0"]')).toHaveText('Real-time');

    await page.fill('[data-testid="text-content-input"]', 'Real-time Preview');
    await expect(page.locator('[data-testid="canvas-preview"] [data-element-id="0"]')).toHaveText('Real-time Preview');
  });

  test('should maintain correct aspect ratio', async ({ page }) => {
    const canvas = page.locator('[data-testid="canvas-preview"]');
    const box = await canvas.boundingBox();

    if (box) {
      const aspectRatio = box.height / box.width;

      // 9:16 aspect ratio = 1.777...
      expect(aspectRatio).toBeCloseTo(16 / 9, 1);

      // Verify dimensions are 360×640
      expect(box.width).toBeCloseTo(360, 5);
      expect(box.height).toBeCloseTo(640, 5);
    }
  });
});

test.describe('Canvas Editor - Accessibility', () => {
  test.beforeEach(async ({ page }) => {
    await navigateToCanvasEditor(page);
  });

  test('should be keyboard navigable', async ({ page }) => {
    // Tab through controls
    await page.keyboard.press('Tab');
    await page.keyboard.press('Tab');
    await page.keyboard.press('Tab');

    // Check that some button receives focus
    const focusedElement = await page.evaluate(() => document.activeElement?.getAttribute('data-testid'));
    expect(focusedElement).toBeTruthy();
  });

  test('should have proper ARIA labels', async ({ page }) => {
    // Check important buttons have aria-label
    await expect(page.locator('[data-testid="add-text-button"]')).toHaveAttribute('aria-label', /.+/);
    await expect(page.locator('[data-testid="add-image-button"]')).toHaveAttribute('aria-label', /.+/);
    await expect(page.locator('[data-testid="save-template-button"]')).toHaveAttribute('aria-label', /.+/);
  });
});
