import { defineStore } from 'pinia'
import { watch, nextTick } from 'vue'
import { useColorMode } from '@vueuse/core'
import { Window } from '@wailsio/runtime'

export type ThemeMode = 'auto' | 'light' | 'dark'

export const useSettingsStore = defineStore('settings', () => {
    const mode = useColorMode({
        emitAuto: true,
        initialValue: 'auto',
    })

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

    // Watch for changes and save to disk
    watch(mode, () => {
        saveSettings()
    })

    return {
        mode,
        loadSettings,
    }
})
