import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount } from '@vue/test-utils'
import { createTestingPinia } from '@pinia/testing'
import TitleBar from "@/components/common/TitleBar.vue"
import { useServicesStore } from "@/stores/services"
import { Window } from '@wailsio/runtime'

vi.mock('@wailsio/runtime', () => ({
  Window: {
    ToggleMaximise: vi.fn(),
    IsMaximised: vi.fn(() => Promise.resolve(false))
  },
  System: {
    IsMac: vi.fn(() => false),
    Environment: vi.fn(() => Promise.resolve({ OS: 'linux' }))
  }
}))

vi.mock('vue-router', () => ({
  useRouter: () => ({
    push: vi.fn(),
  }),
}))

describe('TitleBar.vue', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('renders all sections, search bar, and environment selector', () => {
    const wrapper = mount(TitleBar, {
      global: {
        plugins: [createTestingPinia()],
      },
    })

    // Header
    expect(wrapper.find('header.titlebar').exists()).toBe(true)

    // SearchBar check
    expect(wrapper.find('input[placeholder="Search requests..."]').exists()).toBe(true)

    // EnvironmentSelector check
    expect(wrapper.findComponent({ name: 'EnvironmentSelector' }).exists()).toBe(true)
  })

  it('applies unsafe styling class when active environment is unsafe', async () => {
    const pinia = createTestingPinia({ stubActions: false })
    const servicesStore = useServicesStore()
    servicesStore.services = [
      {
        id: 'service-1',
        name: 'Test Service',
        directory: '/test',
        isAuthenticated: false,
        selectedEnvironment: 'PROD',
        environments: [{ name: 'PROD', isUnsafe: true, variables: [] }],
        endpoints: [],
      },
    ]

    const wrapper = mount(TitleBar, {
      global: {
        plugins: [pinia],
      },
    })

    await wrapper.vm.$nextTick()
    expect(wrapper.find('header.titlebar').classes()).toContain('is-unsafe-titlebar')
  })

  it('calls Wails ToggleMaximise on double-click', async () => {
    const wrapper = mount(TitleBar, {
      global: {
        plugins: [createTestingPinia()],
      },
    })
    await wrapper.find('header.titlebar').trigger('dblclick')
    expect(Window.ToggleMaximise).toHaveBeenCalled()
  })
})


