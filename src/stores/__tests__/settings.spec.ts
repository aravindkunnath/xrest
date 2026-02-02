import { describe, it, expect, vi, beforeEach } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { useSettingsStore } from '../settings'
import { invoke } from '@tauri-apps/api/core'

vi.mock('@tauri-apps/api/core', () => ({
    invoke: vi.fn(),
}))

describe('Settings Store', () => {
    beforeEach(() => {
        setActivePinia(createPinia())
        vi.clearAllMocks()
    })

    it('should load settings correctly', async () => {
        const store = useSettingsStore()
        vi.mocked(invoke).mockResolvedValueOnce({ theme: 'dark' })
        vi.mocked(invoke).mockResolvedValueOnce(undefined) // close_splashscreen

        await store.loadSettings()

        expect(invoke).toHaveBeenCalledWith('get_settings')
        expect(store.mode).toBe('dark')
    })

    it('should save settings when mode changes', async () => {
        const store = useSettingsStore()
        vi.mocked(invoke).mockResolvedValue(undefined)

        store.mode = 'light'

        // Watcher is async-ish, wait for it
        await new Promise(resolve => setTimeout(resolve, 0))

        expect(invoke).toHaveBeenCalledWith('save_settings', {
            settings: { theme: 'light' }
        })
    })
})
