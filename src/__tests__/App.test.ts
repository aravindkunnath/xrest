import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount } from '@vue/test-utils'
import App from '../App.vue'
import { createPinia, setActivePinia } from 'pinia'
import { createRouter, createWebHistory } from 'vue-router'
import { nextTick } from 'vue'

const mockMaximize = vi.fn()
const mockUnmaximize = vi.fn()
const mockIsMaximized = vi.fn(() => Promise.resolve(false))
const mockStartDragging = vi.fn()
const mockShow = vi.fn()

vi.mock('@tauri-apps/api/window', () => ({
    getCurrentWindow: vi.fn(() => ({
        maximize: mockMaximize,
        unmaximize: mockUnmaximize,
        isMaximized: mockIsMaximized,
        startDragging: mockStartDragging,
        show: mockShow
    }))
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

vi.mock('@/stores/settings', () => ({
    useSettingsStore: vi.fn(() => ({
        loadSettings: vi.fn()
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

    it('should call maximize on double click if not maximized', async () => {
        mockIsMaximized.mockResolvedValue(false)
        const wrapper = mount(App, globalOptions)

        const titlebar = wrapper.find('.titlebar')
        await titlebar.trigger('dblclick')

        expect(mockMaximize).toHaveBeenCalled()
    })

    it('should call unmaximize on double click if maximized', async () => {
        mockIsMaximized.mockResolvedValue(true)
        const wrapper = mount(App, globalOptions)

        const titlebar = wrapper.find('.titlebar')
        await titlebar.trigger('dblclick')

        expect(mockUnmaximize).toHaveBeenCalled()
    })

    it('should call startDragging only on single click', async () => {
        const wrapper = mount(App, globalOptions)

        const titlebar = wrapper.find('.titlebar')

        // Test single click (detail 1)
        // We use dispatchEvent because Vue Test Utils trigger has issues setting detail on mousedown in jsdom
        const event1 = new MouseEvent('mousedown', { detail: 1, bubbles: true })
        titlebar.element.dispatchEvent(event1)
        await nextTick()
        expect(mockStartDragging).toHaveBeenCalled()

        // Test double click (detail 2)
        vi.clearAllMocks()
        const event2 = new MouseEvent('mousedown', { detail: 2, bubbles: true })
        titlebar.element.dispatchEvent(event2)
        await nextTick()
        expect(mockStartDragging).not.toHaveBeenCalled()
    })
})
