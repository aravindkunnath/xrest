import { describe, it, expect, beforeEach } from 'vitest'
import { mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import EnvironmentSelector from "@/features/environments/components/EnvironmentSelector.vue"
import { useServicesStore } from "@/stores/services"
import { useTabsStore } from "@/stores/tabs"

describe('EnvironmentSelector.vue', () => {
  let pinia: ReturnType<typeof createPinia>

  const mockServices = [
    {
      id: 'service-1',
      name: 'User Service',
      directory: '/services/user',
      isAuthenticated: false,
      selectedEnvironment: 'DEV',
      environments: [
        { name: 'DEV', isUnsafe: false, variables: [] },
        { name: 'STAGING', isUnsafe: false, variables: [] },
        { name: 'PROD', isUnsafe: true, variables: [] },
      ],
      endpoints: [],
    },
    {
      id: 'service-2',
      name: 'Payment Service',
      directory: '/services/payment',
      isAuthenticated: false,
      selectedEnvironment: 'LIVE_PROD',
      environments: [
        { name: 'SANDBOX', isUnsafe: false, variables: [] },
        { name: 'LIVE_PROD', isUnsafe: true, variables: [] },
      ],
      endpoints: [],
    },
  ]

  beforeEach(() => {
    pinia = createPinia()
    setActivePinia(pinia)
    const servicesStore = useServicesStore()
    servicesStore.services = JSON.parse(JSON.stringify(mockServices))
  })

  it('renders environment select dropdown for the active service', () => {
    const wrapper = mount(EnvironmentSelector, {
      global: {
        plugins: [pinia],
      },
    })

    expect(wrapper.exists()).toBe(true)
    expect(wrapper.text()).toContain('DEV')
  })

  it('updates servicesStore.setSelectedEnvironment when a new environment is selected', async () => {
    const servicesStore = useServicesStore()
    mount(EnvironmentSelector, {
      global: {
        plugins: [pinia],
      },
    })

    await servicesStore.setSelectedEnvironment('service-1', 'PROD')
    expect(servicesStore.services[0].selectedEnvironment).toBe('PROD')
  })

  it('shows unsafe environment warning badge/icon when current environment is unsafe', async () => {
    const servicesStore = useServicesStore()
    const wrapper = mount(EnvironmentSelector, {
      global: {
        plugins: [pinia],
      },
    })

    await servicesStore.setSelectedEnvironment('service-1', 'PROD')
    await wrapper.vm.$nextTick()

    expect(wrapper.classes()).toContain('is-unsafe')
  })

  it('switches active service environment display when activeTab service changes', async () => {
    const tabsStore = useTabsStore()
    tabsStore.tabs = [
      { id: 'tab-1', title: 'User Request', serviceId: 'service-1', type: 'request' },
      { id: 'tab-2', title: 'Payment Request', serviceId: 'service-2', type: 'request' },
    ]
    tabsStore.activeTab = 'tab-2'

    const wrapper = mount(EnvironmentSelector, {
      global: {
        plugins: [pinia],
      },
    })

    expect(wrapper.text()).toContain('LIVE_PROD')
  })
})
