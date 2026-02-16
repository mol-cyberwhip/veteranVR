import { test, expect } from '@playwright/test';
import { setupTauriMock } from '../src/test/mock-tauri';

test.describe('Baseline E2E', () => {
  test.beforeEach(async ({ page }) => {
    page.on('console', msg => console.log(`BROWSER: ${msg.text()}`));
    page.on('pageerror', err => console.log(`BROWSER ERROR: ${err.message}`));
    // Inject the mock before the page loads
    await page.addInitScript(setupTauriMock);
    await page.goto('/');
  });

  test('App Shell loading', async ({ page }) => {
    await expect(page.locator('.app-shell')).toBeVisible();
    await expect(page.locator('.titlebar')).toBeVisible();
    await expect(page.locator('.sidebar')).toBeVisible();
  });

  test('Device connection status', async ({ page }) => {
    // Check if there is an error message
    const errorStatus = page.locator('#device-status');
    const errorText = await errorStatus.textContent();
    console.log('Device Status Text:', errorText);

    if (errorText?.includes('Device state contract error')) {
        throw new Error(`Contract Error: ${errorText}`);
    }

    // Check for the "ready" status indicator which corresponds to a connected device
    const status = page.locator('.sidebar-device-status.connected');
    await expect(status).toBeVisible();
    await expect(status).toHaveText(/Connected/i);
    
    // Check device info in sidebar
    await expect(page.locator('.sidebar-device-info')).toContainText('Mock Device');
  });

  test('Tab navigation', async ({ page }) => {
    // Find tab buttons (assuming they have specific classes or text)
    // Based on index.html analysis, there are sidebar items that act as tabs
    
    // Check "Library" tab
    const libraryTab = page.locator('.sidebar-item', { hasText: 'Library' });
    await expect(libraryTab).toBeVisible();
    
    // Check "Downloads" tab
    const downloadsTab = page.locator('.sidebar-item', { hasText: 'Downloads' });
    await expect(downloadsTab).toBeVisible();

    // Click Downloads
    await downloadsTab.click();
    await expect(downloadsTab).toHaveClass(/active/);
    
    // Click Library
    await libraryTab.click();
    await expect(libraryTab).toHaveClass(/active/);
  });
  
  test('Catalog syncing', async ({ page }) => {
      // The mock returns a catalog status with game_count: 5.
      // We should check if this is reflected in the UI.
      // This depends on how the UI renders it. 
      // Assuming there is some status text or game list.
      
      // We can also check if backend_catalog_status was called by spying on the console logs
      // but strictly speaking we want to verify the UI.
      
      // If the catalog is "synced", the UI might show "Ready" in the status bar.
      const readyStatus = page.locator('.status.ready');
      await expect(readyStatus).toBeVisible();
      await expect(readyStatus).toHaveText('Desktop Ready');
  });
});
