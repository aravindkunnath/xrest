import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount } from '@vue/test-utils'
import App from '@/App.vue'
import { createPinia, setActivePinia } from 'pinia'
import { createRouter, createWebHistory } from 'vue-router'

vi.mock('@wailsio/runtime', () => ({
    Window: {
        ToggleMaximise: vi.fn(),
        IsMaximised: vi.fn(() => Promise.resolve(false))
    }
}))

const router = createRouter({
    history: createWebHistory(),
    routes: [
        { path: '/', component: { template: '<div>Home</div>' } },
        { path: '/services', component: { template: '<div>Services</div>' } },
        { path: '/collections', component: { template: '<div>Collections</div>' } },
        { path: '/secrets', component: { template: '<div>Secrets</div>' } },
        { path: '/history', component: { template: '<div>History</div>' } },
        { path: '/settings', component: { template: '<div>Settings</div>' } }
    ]
})

const mockLoadSettings = vi.fn()

vi.mock('@/stores/settings', () => ({
    useSettingsStore: vi.fn(() => ({
        loadSettings: mockLoadSettings
    }))
}))

vi.mock('lucide-vue-next', () => ({
    History: { template: '<span>history</span>' },
    Layers: { template: '<span>layers</span>' },
    LayoutGrid: { template: '<span>grid</span>' },
    Settings: { template: '<span>settings</span>' },
    Key: { template: '<span>key</span>' }
}))

describe('App', () => {
    beforeEach(() => {
        setActivePinia(createPinia())
        vi.clearAllMocks()
    })

    const globalOptions = {
        global: {
            plugins: [router],
            stubs: {
                MainLayout: { template: '<div><slot /></div>' }
            }
        }
    }

    it('renders TitleBar and main layout', () => {
        const wrapper = mount(App, globalOptions)
        expect(wrapper.find('.app-container').exists()).toBe(true)
        expect(wrapper.find('header.titlebar').exists()).toBe(true)
    })

    it('loads settings on mount', () => {
        mount(App, globalOptions)
        expect(mockLoadSettings).toHaveBeenCalled()
    })
})
