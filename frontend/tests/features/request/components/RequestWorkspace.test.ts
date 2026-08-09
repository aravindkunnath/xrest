import { mount, flushPromises } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, afterEach, describe, expect, it, vi } from 'vitest'
import { ref } from 'vue'
import RequestWorkspace from "@/features/request/components/RequestWorkspace.vue"
import RequestHistory from "@/features/history/components/RequestHistory.vue"
import { useVersionsStore } from "@/stores/versions"
import { useTabsStore } from "@/stores/tabs"

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

vi.mock("@/composables/useTabManager", () => ({
    useTabManager: vi.fn(() => ({
        tabs: mockTabs,
        activeTab: mockActiveTab,
        addTab: mockAddTab,
        closeTab: mockCloseTab,
        updateTabSnapshot: mockUpdateTabSnapshot
    }))
}))

vi.mock("@/features/request/composables/useEnvironmentVariables", () => ({
    useEnvironmentVariables: vi.fn(() => ({
        getTabVariables: vi.fn(() => ({})),
        getEnvName: vi.fn(() => 'DEV'),
        isUnsafeEnv: vi.fn(() => false)
    }))
}))

vi.mock("@/features/request/composables/useRequestExecution", () => ({
    useRequestExecution: vi.fn(() => ({
        isSending: ref(false),
        handleSendRequest: vi.fn()
    }))
}))

vi.mock("@/composables/useDialogState", () => ({
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
vi.mock("@/features/services/composables/useServiceSettings", () => ({
    useServiceSettings: vi.fn(() => ({
        saveSettings: mockSaveSettings
    }))
}))

// Track mounted wrappers so their window keydown listeners are removed between
// tests (RequestWorkspace registers a native listener in onMounted).
let mountedWrappers: ReturnType<typeof mount>[] = []

function mountWorkspace(options: Parameters<typeof mount>[0] = {}) {
    const wrapper = mount(RequestWorkspace, options)
    mountedWrappers.push(wrapper)
    return wrapper
}

describe('RequestWorkspace', () => {
    beforeEach(() => {
        setActivePinia(createPinia())
        localStorage.clear()
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

    afterEach(() => {
        mountedWrappers.forEach((w) => w.unmount())
        mountedWrappers = []
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
        const wrapper = mountWorkspace({
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
        const wrapper = mountWorkspace({
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
        const wrapper = mountWorkspace({
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

        // Saving now asynchronously records a version through the store first.
        await flushPromises()

        expect(wrapper.emitted('save-request')).toBeTruthy()
        expect(wrapper.emitted('save-request')![0][0]).toMatchObject({
            serviceIndex: 0,
            tab: mockTabs.value[0]
        })
        expect(mockUpdateTabSnapshot).toHaveBeenCalledWith(mockTabs.value[0])
    })

    it('should record a version through the store, bump lastVersion, and drop versions from the payload', async () => {
        const versionsStore = useVersionsStore()

        const wrapper = mountWorkspace({
            props: {
                items: [
                    {
                        id: 'service-1',
                        endpoints: [
                            {
                                id: 'endpoint-1',
                                name: 'Endpoint 1',
                                method: 'GET',
                                url: '/old',
                                lastVersion: 3,
                                versions: [{ version: 1, config: {}, lastUpdated: 0 }],
                            }
                        ],
                        environments: []
                    }
                ]
            },
            global: globalOptions
        })

        const event = new KeyboardEvent('keydown', { key: 's', ctrlKey: true, bubbles: true })
        window.dispatchEvent(event)
        await flushPromises()

        const emitted = wrapper.emitted('save-request')
        expect(emitted).toBeTruthy()
        const payload = emitted![0][0]
        expect(payload.serviceIndex).toBe(0)

        const updatedEndpoint = payload.updatedItem.endpoints[0]
        expect(updatedEndpoint).not.toHaveProperty('versions')
        expect(updatedEndpoint.lastVersion).toBe(4)

        // Version was recorded in the versions store (mock SQLite) and bumps the badge count.
        expect(versionsStore.getCount('endpoint-1')).toBe(1)
        expect(versionsStore.getEntries('endpoint-1')[0]).toMatchObject({
            version: 1,
            config: { method: 'GET', url: 'http://test.com' },
        })
    })

    it('should render the version-count badge from the versions store', async () => {
        const versionsStore = useVersionsStore()
        await versionsStore.addVersion('service-1', 'endpoint-1', { method: 'GET', url: '/a' } as any, 50)
        await versionsStore.addVersion('service-1', 'endpoint-1', { method: 'GET', url: '/b' } as any, 50)

        // The panels must render their slots so the inner request tabs (incl.
        // Versions) are reachable.
        const globalOptionsWithSlots = {
            ...globalOptions,
            stubs: {
                ...globalOptions.stubs,
                ResizablePanelGroup: { template: '<div><slot /></div>' },
                ResizablePanel: { template: '<div><slot /></div>' },
            },
        }

        const wrapper = mountWorkspace({
            props: { items: [] },
            global: globalOptionsWithSlots,
        })

        // The Versions trigger badge number equals the store count.
        const badge = wrapper
            .findAll('button.tabs-trigger')
            .map((w) => w.text())
            .find((text) => text.includes('Versions'))
        expect(badge).toBeTruthy()
        expect(badge).toContain('2')
    })

    it('RESTORE: a version loaded from (mock) SQLite restores method/url/params/headers/body/preflight on the tab', async () => {
        const versionsStore = useVersionsStore()
        const tabsStore = useTabsStore()

        const storedConfig = {
            method: 'POST',
            url: 'http://restored.example.com/items',
            authenticated: false,
            authType: 'none',
            params: [
                { name: 'page', value: '2', enabled: true },
                { name: 'limit', value: '10', enabled: true },
            ],
            headers: [
                { name: 'Accept', value: 'application/json', enabled: true },
            ],
            body: '{"q":"hello"}',
            preflight: {
                enabled: true,
                method: 'POST',
                url: 'https://auth.example.com/token',
                body: '{}',
                tokenKey: 'access_token',
                tokenHeader: 'Authorization',
            },
        }
        await versionsStore.addVersion('service-1', 'restore-1', storedConfig, 50)

        tabsStore.tabs.push({
            id: 'restore-tab',
            title: 'Restore',
            type: 'request',
            method: 'GET',
            url: 'http://old.example.com/',
            params: [],
            headers: [],
            body: { content: 'old' },
            preflight: { enabled: false },
            serviceId: 'service-1',
            endpointId: 'restore-1',
        })

        const wrapper = mount(RequestHistory, {
            props: {
                tabId: 'restore-tab',
                serviceId: 'service-1',
                endpointId: 'restore-1',
            },
            global: {
                stubs: {
                    Accordion: { template: '<div><slot /></div>' },
                    AccordionItem: { template: '<div><slot /></div>' },
                    AccordionTrigger: { template: '<div><slot /></div>' },
                    AccordionContent: { template: '<div><slot /></div>' },
                    Button: {
                        template: '<button @click="$emit(\'click\')"><slot /></button>',
                    },
                    AlertCircle: { template: '<span />' },
                    ArrowRight: { template: '<span />' },
                    ArrowUpRight: { template: '<span />' },
                    Layers: { template: '<span />' },
                    Minus: { template: '<span />' },
                    Plus: { template: '<span />' },
                    RefreshCw: { template: '<span />' },
                },
            },
        })
        mountedWrappers.push(wrapper)

        await flushPromises()

        const restoreButton = wrapper
            .findAll('button')
            .find((b) => b.text().includes('RESTORE THIS VERSION'))
        expect(restoreButton).toBeTruthy()
        await restoreButton!.trigger('click')

        const tab = tabsStore.tabs.find((t) => t.id === 'restore-tab') as any
        expect(tab).toBeTruthy()
        expect(tab.method).toBe(storedConfig.method)
        expect(tab.url).toBe(storedConfig.url)
        expect(tab.params).toEqual(storedConfig.params)
        expect(tab.headers).toEqual(storedConfig.headers)
        expect(tab.body.content).toBe(storedConfig.body)
        expect(tab.preflight).toMatchObject(storedConfig.preflight)
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

        mountWorkspace({
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
