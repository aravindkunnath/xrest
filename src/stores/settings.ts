import { defineStore } from 'pinia'
import { watch, nextTick } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useColorMode } from '@vueuse/core'

export type ThemeMode = 'auto' | 'light' | 'dark'

export const useSettingsStore = defineStore('settings', () => {
    const mode = useColorMode({
        emitAuto: true,
        initialValue: 'auto',
    })

    const loadSettings = async () => {
        try {
            console.log('Loading settings...')
            // Use the new get_settings command
            const settings = await invoke<{ theme: string }>('get_settings')
            if (settings?.theme === 'system') {
                mode.value = 'auto'
            } else if (settings?.theme) {
                mode.value = settings.theme as any
            }
        } catch (error) {
            console.error('Failed to load settings:', error)
        } finally {
            // Apply theme classes
            await nextTick()

            // Short delay for smoothness
            await new Promise(resolve => setTimeout(resolve, 800))

            // Use the refactored close_splashscreen command
            try {
                await invoke('close_splashscreen')
            } catch (e) {
                console.error('Failed to close splashscreen via Rust:', e)
                // JS Fallback
                try {
                    const { getCurrentWindow } = await import('@tauri-apps/api/window')
                    await getCurrentWindow().show()
                } catch (inner) { }
            }
        }
    }

    const saveSettings = async () => {
        try {
            const themeToSave = mode.value === 'auto' ? 'system' : mode.value
            // Use the new save_settings command with correct payload
            await invoke('save_settings', { settings: { theme: themeToSave } })
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
