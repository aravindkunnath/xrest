import { Builder, WebDriver, Capabilities } from 'selenium-webdriver';
import { describe, it, expect, beforeAll, afterAll } from 'vitest';
import { fileURLToPath } from 'url';
import path from 'path';
import { spawn, ChildProcess } from 'child_process';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const APP_PATH = path.resolve(__dirname, '../../src-tauri/target/debug/xrest');

describe('Application Load', () => {
    let driver: WebDriver;
    let tauriDriver: ChildProcess;

    beforeAll(async () => {
        // Start tauri-driver
        tauriDriver = spawn('tauri-driver', [], {
            stdio: 'inherit',
        });

        // Wait for tauri-driver to start
        await new Promise((resolve) => setTimeout(resolve, 2000));

        const capabilities = new Capabilities();
        capabilities.set('tauri:options', {
            application: APP_PATH,
        });
        capabilities.setBrowserName('wry');

        driver = await new Builder()
            .withCapabilities(capabilities)
            .usingServer('http://127.0.0.1:4444/')
            .build();
    }, 60000); // 60 second timeout for beforeAll

    afterAll(async () => {
        if (driver) {
            await driver.quit();
        }
        if (tauriDriver) {
            tauriDriver.kill();
        }
    });

    it('should load the application and show the main window', async () => {
        // Check if we can get the title or some element
        const title = await driver.getTitle();
        console.log('App title:', title);

        // In Tauri, the title might be what's in tauri.conf.json or the HTML title
        // Let's just check if it's not empty
        expect(title).toBeTruthy();
        expect(title.length).toBeGreaterThan(0);
    }, 30000); // 30 second timeout for this test

    it('should have the root element', async () => {
        const root = await driver.executeScript('return document.getElementById("app")');
        expect(root).toBeTruthy();
    }, 30000); // 30 second timeout for this test
});
