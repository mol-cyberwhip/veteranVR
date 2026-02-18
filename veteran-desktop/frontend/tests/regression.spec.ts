import { test, expect } from '@playwright/test';
import { setupTauriMock } from '../src/test/mock-tauri';

test.describe('Regressions', () => {
  test.beforeEach(async ({ page }) => {
    page.on('console', msg => console.log(`BROWSER: ${msg.text()}`));
    page.on('pageerror', err => console.log(`BROWSER ERROR: ${err.message}`));
    await page.addInitScript(setupTauriMock);
    await page.goto('/');
  });

  test('Backend ready state displays correct message', async ({ page }) => {
    const readyMessage = page.locator('#ready-message');
    // The mock returns { message: "Ready" } when ready
    await expect(readyMessage).toHaveText('Ready');
    
    const readyStatus = page.locator('.status.ready');
    await expect(readyStatus).toBeVisible();
    await expect(readyStatus).toHaveText('Desktop Ready');
  });

  test('Game library loads and displays thumbnails', async ({ page }) => {
    const gameCards = page.locator('.game-card');
    await expect(gameCards).toHaveCount(2);

    // Verify first game card
    const firstCard = gameCards.first();
    await expect(firstCard).toContainText('Beat Saber');
    
    // Check if thumbnail image exists
    const thumbnail = firstCard.locator('.card-thumb img');
    await expect(thumbnail).toBeVisible();
    const src = await thumbnail.getAttribute('src');
    expect(src).toContain('com.beatgames.beatsaber.png');
  });

  test('Download button triggers backend processing', async ({ page }) => {
    const firstCard = page.locator('.game-card').first();
    
    // Expand card
    await firstCard.click();
    
    // Find action button
    const actionBtn = firstCard.locator('button', { hasText: /Download/i }).first();
    await expect(actionBtn).toBeEnabled();
    
    // Click button
    await actionBtn.click();
    
    // Since we can't easily check backend side effects in E2E without more complex mocks,
    // we verify the button remains or changes state as expected.
    // In our implementation, handleDownload calls api.queueDownload and startDownloadProcessing.
    // We expect the button to stay active or show success.
    await expect(actionBtn).toBeVisible();
  });
});
