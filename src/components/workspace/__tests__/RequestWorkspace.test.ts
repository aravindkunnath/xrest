import { mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { ref } from 'vue'
import RequestWorkspace from '../RequestWorkspace.vue'

// Mock composables
const mockTabs = ref([
    {
        id: 'tab-1',
        title: 'Request 1',
        type: 'request',
        method: 'GET',
        url: 'http://test.com',
        params: [],
        headers: [],
        body: { content: '' },
        endpointId: 'endpoint-1',
        serviceId: 'service-1',
        preflight: { enabled: false },
        response: { status: 0, statusText: '', time: '0ms', size: '0 B', type: '', body: '', error: '', headers: [], requestHeaders: [] },
        versions: []
    }
])
const mockActiveTab = ref('tab-1')
const mockCloseTab = vi.fn()
const mockAddTab = vi.fn()
const mockUpdateTabSnapshot = vi.fn()

vi.mock('@/composables/useTabManager', () => ({
    useTabManager: vi.fn(() => ({
        tabs: mockTabs,
        activeTab: mockActiveTab,
        addTab: mockAddTab,
        closeTab: mockCloseTab,
        updateTabSnapshot: mockUpdateTabSnapshot
    }))
}))

vi.mock('@/composables/useEnvironmentVariables', () => ({
    useEnvironmentVariables: vi.fn(() => ({
        getTabVariables: vi.fn(() => ({})),
        getEnvName: vi.fn(() => 'DEV'),
        isUnsafeEnv: vi.fn(() => false)
    }))
}))

vi.mock('@/composables/useRequestExecution', () => ({
    useRequestExecution: vi.fn(() => ({
        isSending: ref(false),
        handleSendRequest: vi.fn()
    }))
}))

vi.mock('@/composables/useDialogState', () => ({
    useDialogState: vi.fn(() => ({
        isUnsafeDialogOpen: ref(false)
    }))
}))

vi.mock('vue-sonner', () => ({
    toast: {
        error: vi.fn(),
        success: vi.fn()
    }
}))

const mockSaveSettings = vi.fn()
vi.mock('@/composables/useServiceSettings', () => ({
    useServiceSettings: vi.fn(() => ({
        saveSettings: mockSaveSettings
    }))
}))

describe('RequestWorkspace', () => {
    beforeEach(() => {
        setActivePinia(createPinia())
        vi.clearAllMocks()
        mockTabs.value = [
            {
                id: 'tab-1',
                title: 'Request 1',
                type: 'request',
                method: 'GET',
                url: 'http://test.com',
                params: [],
                headers: [],
                body: { content: '' },
                endpointId: 'endpoint-1',
                serviceId: 'service-1',
                preflight: { enabled: false },
                response: { status: 0, statusText: '', time: '0ms', size: '0 B', type: '', body: '', error: '', headers: [], requestHeaders: [] },
                versions: []
            }
        ]
        mockActiveTab.value = 'tab-1'
    })

    const globalOptions = {
        stubs: {
            Tabs: { template: '<div><slot /></div>' },
            TabsContent: { template: '<div><slot /></div>' },
            TabsList: { template: '<div><slot /></div>' },
            TabsTrigger: {
                template: '<button class="tabs-trigger" @mousedown.middle="$emit(\'mousedown.middle\')" @auxclick.middle="$emit(\'auxclick.middle\')"><slot /></button>'
            },
            RequestUrlBar: true,
            RequestParameters: true,
            RequestBody: true,
            RequestHistory: true,
            ResponseViewer: true,
            ServiceSettingsView: true,
            ResizablePanelGroup: true,
            ResizablePanel: true,
            ResizableHandle: true,
            Settings2: { template: '<span>icon</span>' },
            X: { template: '<span>x</span>' },
            Plus: { template: '<span>+</span>' },
            Play: { template: '<span>play</span>' }
        }
    }

    it('should call closeTab on middle click (mousedown)', async () => {
        const wrapper = mount(RequestWorkspace, {
            props: {
                items: []
            },
            global: globalOptions
        })

        const trigger = wrapper.find('.tabs-trigger')
        await trigger.trigger('mousedown', { button: 1 })

        expect(mockCloseTab).toHaveBeenCalledWith('tab-1')
    })

    it('should call closeTab on middle click (auxclick)', async () => {
        const wrapper = mount(RequestWorkspace, {
            props: {
                items: []
            },
            global: globalOptions
        })

        const trigger = wrapper.find('.tabs-trigger')
        await trigger.trigger('auxclick', { button: 1 })

        expect(mockCloseTab).toHaveBeenCalledWith('tab-1')
    })

    it('should call handleSaveRequest and emit save-request on Ctrl+S', async () => {
        const wrapper = mount(RequestWorkspace, {
            props: {
                items: [
                    {
                        id: 'service-1',
                        endpoints: [{ id: 'endpoint-1', name: 'Endpoint 1' }],
                        environments: []
                    }
                ]
            },
            global: globalOptions
        })

        // Dispatch global keydown event
        const event = new KeyboardEvent('keydown', {
            key: 's',
            ctrlKey: true,
            bubbles: true
        })
        window.dispatchEvent(event)

        expect(wrapper.emitted('save-request')).toBeTruthy()
        expect(wrapper.emitted('save-request')![0][0]).toMatchObject({
            serviceIndex: 0,
            tab: mockTabs.value[0]
        })
        expect(mockUpdateTabSnapshot).toHaveBeenCalledWith(mockTabs.value[0])
    })

    it('should call handleUpdateSettings and emit update-item on Ctrl+S for settings tab', async () => {
        mockTabs.value = [{
            id: 'tab-settings',
            title: 'Settings',
            type: 'settings',
            serviceId: 'service-1',
            serviceData: { id: 'service-1', name: 'Service 1' }
        } as any]
        mockActiveTab.value = 'tab-settings'

        mount(RequestWorkspace, {
            props: {
                items: [
                    {
                        id: 'service-1',
                        name: 'Service 1'
                    }
                ]
            },
            global: globalOptions
        })

        const event = new KeyboardEvent('keydown', {
            key: 's',
            metaKey: true, // Test Cmd+S (mac)
            bubbles: true
        })
        window.dispatchEvent(event)

        expect(mockSaveSettings).toHaveBeenCalledWith(mockTabs.value[0])
    })
})
