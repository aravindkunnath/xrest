import { describe, it, expect, beforeEach, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import ServiceSettingsView from '../ServiceSettingsView.vue'
import { createPinia, setActivePinia } from 'pinia'

// Mock secrets store
vi.mock('@/stores/secrets', () => ({
    useSecretsStore: vi.fn(() => ({
        fetchSecrets: vi.fn(),
        secrets: []
    }))
}))

const mockReloadAll = vi.fn()

vi.mock('@/composables/useServiceSettings', () => ({
    useServiceSettings: vi.fn(() => ({
        saveSettings: vi.fn(),
        deleteItem: vi.fn(),
        reloadAll: mockReloadAll,
        syncGit: vi.fn(),
        initGit: vi.fn()
    }))
}))

describe('ServiceSettingsView - Reload Button', () => {
    const createMockServiceTab = () => ({
        id: 'settings-test',
        type: 'settings',
        serviceId: 'service-1',
        title: 'Test Service',
        serviceData: {
            id: 'service-1',
            name: 'Test Service',
            directory: '/test/path',
            isAuthenticated: false,
            authType: 'none',
            endpoints: [],
            environments: [
                {
                    name: 'DEV',
                    isUnsafe: false,
                    variables: [{ name: 'BASE_URL', value: 'http://localhost:3000' }]
                }
            ]
        }
    })

    const createMockCollectionTab = () => ({
        id: 'settings-test',
        type: 'settings',
        serviceId: 'collection-1',
        title: 'Test Collection',
        serviceData: {
            id: 'collection-1',
            name: 'Test Collection',
            directory: '', // Collections don't have directories
            isAuthenticated: false,
            authType: 'none',
            endpoints: [],
            environments: [
                {
                    name: 'GLOBAL',
                    isUnsafe: false,
                    variables: [{ name: 'BASE_URL', value: 'http://localhost:3000' }]
                }
            ]
        }
    })

    beforeEach(() => {
        setActivePinia(createPinia())
        vi.clearAllMocks()
    })

    const mountOptions = {
        global: {
            stubs: {
                RequestAuth: true,
                Popover: true,
                PopoverContent: true,
                PopoverTrigger: true,
                Table: true,
                TableHeader: true,
                TableBody: true,
                TableRow: true,
                TableHead: true,
                TableCell: true,
                Button: { template: '<button><slot /></button>' },
                Input: true,
                Label: true,
                Switch: true,
                ShieldCheck: true,
                Settings2: true,
                Globe: true,
                Trash2: true,
                Save: true,
                RefreshCw: true,
                GitBranch: true,
                CheckCircle2: true,
                Plus: true
            }
        }
    }

    it('should show reload button for services with directories', async () => {
        const tab = createMockServiceTab()
        const wrapper = mount(ServiceSettingsView, {
            props: { tab, gitStatus: null },
            ...mountOptions
        })

        const reloadButton = wrapper.findAll('button').find(b => b.text().includes('Reload'))
        expect(reloadButton?.exists()).toBe(true)
    })

    it('should NOT show reload button for collections (no directory)', async () => {
        const tab = createMockCollectionTab()
        const wrapper = mount(ServiceSettingsView, {
            props: { tab, gitStatus: null },
            ...mountOptions
        })

        const reloadButton = wrapper.findAll('button').find(b => b.text().includes('Reload'))
        expect(reloadButton).toBeUndefined()
    })

    it('should emit reload event when reload button is clicked', async () => {
        const tab = createMockServiceTab()
        const wrapper = mount(ServiceSettingsView, {
            props: { tab, gitStatus: null },
            ...mountOptions
        })

        const reloadButton = wrapper.findAll('button').find(b => b.text().includes('Reload'))
        await reloadButton?.trigger('click')

        expect(mockReloadAll).toHaveBeenCalled()
    })
})
