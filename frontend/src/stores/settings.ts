import { defineStore } from 'pinia'
import { watch, nextTick, ref } from 'vue'
import { useColorMode } from '@vueuse/core'
import { Window } from '@wailsio/runtime'
import { LoadZoomLevel, SaveZoomLevel } from '../../bindings/xrest/cmd/wails/settingsgateway'

export type ThemeMode = 'auto' | 'light' | 'dark'

export const MIN_VERSIONS_LIMIT = 5
export const MAX_VERSIONS_LIMIT = 50
export const VERSIONS_LIMIT_STEP = 5
export const DEFAULT_VERSIONS_LIMIT = 50

// Clamps to the 5..50 range and snaps to the nearest step of 5.
export function normalizeVersionsLimit(value: unknown): number {
    const n = Number(value)
    if (!Number.isFinite(n)) return DEFAULT_VERSIONS_LIMIT
    const clamped = Math.min(MAX_VERSIONS_LIMIT, Math.max(MIN_VERSIONS_LIMIT, n))
    return Math.round(clamped / VERSIONS_LIMIT_STEP) * VERSIONS_LIMIT_STEP
}

export const useSettingsStore = defineStore('settings', () => {
    const mode = useColorMode({
        emitAuto: true,
        initialValue: 'auto',
    })

    const layout = ref<'horizontal' | 'vertical'>('horizontal')
    const zoomLevel = ref(0)
    const versionsLimit = ref(DEFAULT_VERSIONS_LIMIT)

    const setVersionsLimit = (value: number) => {
        if (!Number.isFinite(value)) return
        versionsLimit.value = normalizeVersionsLimit(value)
        saveSettings()
    }

    const applyZoom = (level: number) => {
        // Base is 14px. Each zoom level adjusts by 1px (or custom factor like 1.5px)
        const baseSize = 14
        const newSize = baseSize + level * 1
        document.documentElement.style.fontSize = `${newSize}px`
    }

    const loadSettings = async () => {
        try {
            console.log('Loading settings...')
            const saved = localStorage.getItem('xrest_settings')
            if (saved) {
                const settings = JSON.parse(saved)
                if (settings?.theme === 'system') {
                    mode.value = 'auto'
                } else if (settings?.theme) {
                    mode.value = settings.theme as any
                }
                if (settings?.layout) {
                    layout.value = settings.layout as any
                }
                if (settings?.versionsLimit !== undefined && settings?.versionsLimit !== null) {
                    versionsLimit.value = normalizeVersionsLimit(settings.versionsLimit)
                }
            }

            // Load zoom level from Go backend config.yaml
            try {
                const level = await LoadZoomLevel()
                zoomLevel.value = level
                applyZoom(level)
            } catch (err) {
                console.error('Failed to load zoom level:', err)
            }
        } catch (error) {
            console.error('Failed to load settings:', error)
        } finally {
            // Apply theme classes
            await nextTick()

            // Short delay for smoothness
            await new Promise(resolve => setTimeout(resolve, 800))

            // Show window
            // @ts-ignore
            if (window.wails) {
                try {
                    await Window.Show()
                } catch (e) {
                    console.error('Failed to show Wails window:', e)
                }
            }
        }
    }

    const saveSettings = async () => {
        try {
            const themeToSave = mode.value === 'auto' ? 'system' : mode.value
            localStorage.setItem('xrest_settings', JSON.stringify({ theme: themeToSave, layout: layout.value, versionsLimit: versionsLimit.value }))
        } catch (error) {
            console.error('Failed to save settings:', error)
        }
    }

    const setZoomLevel = async (level: number) => {
        if (level < -2 || level > 5) return
        zoomLevel.value = level
        applyZoom(level)
        try {
            await SaveZoomLevel(level)
        } catch (err) {
            console.error('Failed to save zoom level:', err)
        }
    }

    // Watch for changes and save to disk
    watch([mode, layout, versionsLimit], () => {
        saveSettings()
    })

    return {
        mode,
        zoomLevel,
        layout,
        versionsLimit,
        loadSettings,
        setZoomLevel,
        setVersionsLimit,
    }
})

