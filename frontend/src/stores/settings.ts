import { defineStore } from 'pinia'
import { watch, nextTick, ref } from 'vue'
import { useColorMode } from '@vueuse/core'
import { Window } from '@wailsio/runtime'
import { LoadZoomLevel, SaveZoomLevel } from '../../bindings/xrest/cmd/wails/settingsgateway'

export type ThemeMode = 'auto' | 'light' | 'dark'

export const useSettingsStore = defineStore('settings', () => {
    const mode = useColorMode({
        emitAuto: true,
        initialValue: 'auto',
    })

    const zoomLevel = ref(0)

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
            localStorage.setItem('xrest_settings', JSON.stringify({ theme: themeToSave }))
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
    watch(mode, () => {
        saveSettings()
    })

    return {
        mode,
        zoomLevel,
        loadSettings,
        setZoomLevel,
    }
})

