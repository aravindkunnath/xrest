import { describe, it, expect, beforeEach, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import ServiceSettingsView from '../ServiceSettingsView.vue'
import { createPinia, setActivePinia } from 'pinia'

// Mock secrets store to avoid Pinia error if it's not detected properly
vi.mock('@/stores/secrets', () => ({
    useSecretsStore: vi.fn(() => ({
        fetchSecrets: vi.fn(),
        secrets: []
    }))
}))

const mockSaveSettings = vi.fn()
const mockDeleteItem = vi.fn()
const mockReloadAll = vi.fn()

vi.mock('@/composables/useServiceSettings', () => ({
    useServiceSettings: vi.fn(() => ({
        saveSettings: mockSaveSettings,
        deleteItem: mockDeleteItem,
        reloadAll: mockReloadAll,
        syncGit: vi.fn(),
        initGit: vi.fn()
    }))
}))

describe('ServiceSettingsView - isUnsafe Flag', () => {
    const createMockTab = (isUnsafe = false) => ({
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
                    variables: [
                        { name: 'BASE_URL', value: 'http://localhost:3000' }
                    ]
                },
                {
                    name: 'STAGING',
                    isUnsafe: isUnsafe,
                    variables: [
                        { name: 'BASE_URL', value: 'https://staging.example.com' }
                    ]
                },
                {
                    name: 'PRODUCTION',
                    isUnsafe: true,
                    variables: [
                        { name: 'BASE_URL', value: 'https://api.example.com' }
                    ]
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
                Table: { template: '<table><slot /></table>' },
                TableHeader: { template: '<thead><slot /></thead>' },
                TableBody: { template: '<tbody><slot /></tbody>' },
                TableRow: { template: '<tr><slot /></tr>' },
                TableHead: { template: '<th><slot /></th>' },
                TableCell: { template: '<td><slot /></td>' },
                Button: { template: '<button><slot /></button>' },
                Input: { template: '<input />' },
                Label: { template: '<label><slot /></label>' },
                Switch: {
                    template: '<button class="switch" @click="$emit(\'update:modelValue\', !modelValue)"></button>',
                    props: ['modelValue']
                },
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

    it('should display isUnsafe flag correctly for each environment', async () => {
        const tab = createMockTab(false)
        const wrapper = mount(ServiceSettingsView, {
            props: {
                tab,
                gitStatus: null
            },
            ...mountOptions
        })

        // Find all indicators
        const indicators = wrapper.findAll('span[class*="w-1.5 h-1.5"]')

        // DEV indicator (index 0)
        expect(indicators[0].classes()).toContain('bg-green-500')

        // STAGING indicator (index 1)
        expect(indicators[1].classes()).toContain('bg-green-500')

        // PRODUCTION indicator (index 2)
        expect(indicators[2].classes()).toContain('bg-destructive')
    })

    it('should toggle isUnsafe flag when switch is clicked', async () => {
        const tab = createMockTab(false)
        const wrapper = mount(ServiceSettingsView, {
            props: {
                tab,
                gitStatus: null
            },
            ...mountOptions
        })

        // Get the STAGING environment (index 1)
        expect(tab.serviceData.environments[1].isUnsafe).toBe(false)

        const stagingSwitch = wrapper.findAll('.switch')[1]
        await stagingSwitch.trigger('click')

        expect(tab.serviceData.environments[1].isUnsafe).toBe(true)
    })

    it('should preserve isUnsafe flag when saving', async () => {
        const tab = createMockTab(true)
        const wrapper = mount(ServiceSettingsView, {
            props: {
                tab,
                gitStatus: null
            },
            ...mountOptions
        })

        const saveButton = wrapper.findAll('button').find(b => b.text().includes('Save Changes'))
        await saveButton?.trigger('click')

        expect(mockSaveSettings).toHaveBeenCalled()
    })

    it('should display "Prod Warn" label for each environment', async () => {
        const tab = createMockTab()
        const wrapper = mount(ServiceSettingsView, {
            props: {
                tab,
                gitStatus: null
            },
            ...mountOptions
        })

        const prodWarnLabels = wrapper.findAll('span').filter(s => s.text().includes('Unsafe'))
        expect(prodWarnLabels.length).toBeGreaterThanOrEqual(3)
    })
})
