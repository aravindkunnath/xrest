import { describe, it, expect, vi, beforeEach } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { useSettingsStore } from '@/stores/settings'

describe('Settings Store', () => {
    beforeEach(() => {
        setActivePinia(createPinia())
        localStorage.clear()
        vi.clearAllMocks()
    })

    it('should load settings correctly', async () => {
        const store = useSettingsStore()
        localStorage.setItem('xrest_settings', JSON.stringify({ theme: 'dark' }))

        await store.loadSettings()

        expect(store.mode).toBe('dark')
    })

    it('should save settings when mode changes', async () => {
        const store = useSettingsStore()

        store.mode = 'light'

        // Watcher is async-ish, wait for it
        await new Promise(resolve => setTimeout(resolve, 0))

        const saved = localStorage.getItem('xrest_settings')
        expect(saved).toBe(JSON.stringify({ theme: 'light' }))
    })
})
