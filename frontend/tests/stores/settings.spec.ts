import { describe, it, expect, vi, beforeEach } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { useSettingsStore, normalizeVersionsLimit, DEFAULT_VERSIONS_LIMIT, MIN_VERSIONS_LIMIT, MAX_VERSIONS_LIMIT } from "@/stores/settings"

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

    it('should load the versionsLimit from persisted settings', async () => {
        const store = useSettingsStore()
        localStorage.setItem('xrest_settings', JSON.stringify({ versionsLimit: 20 }))

        await store.loadSettings()

        expect(store.versionsLimit).toBe(20)
    })

    it('should default versionsLimit to 50 when unset', async () => {
        const store = useSettingsStore()
        await store.loadSettings()
        expect(store.versionsLimit).toBe(DEFAULT_VERSIONS_LIMIT)
    })

    it('should save settings when mode changes', async () => {
        const store = useSettingsStore()

        store.mode = 'light'

        // Watcher is async-ish, wait for it
        await new Promise(resolve => setTimeout(resolve, 0))

        const saved = localStorage.getItem('xrest_settings')
        expect(saved).toBe(JSON.stringify({ theme: 'light', layout: 'horizontal', versionsLimit: 50 }))
    })

    it('should persist versionsLimit when set', async () => {
        const store = useSettingsStore()
        store.setVersionsLimit(30)

        const saved = JSON.parse(localStorage.getItem('xrest_settings') || '{}')
        expect(saved.versionsLimit).toBe(30)
        expect(store.versionsLimit).toBe(30)
    })

    it.each([
        [3, MIN_VERSIONS_LIMIT],
        [60, MAX_VERSIONS_LIMIT],
        [7, 5],
        [48, 50],
        [12, 10],
        [17, 15],
        ['12', 10],
        [NaN, DEFAULT_VERSIONS_LIMIT],
    ])('normalizeVersionsLimit(%s) -> %s', (input, expected) => {
        expect(normalizeVersionsLimit(input)).toBe(expected)
    })
})
