import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount } from '@vue/test-utils'
import TitleBar from '@/components/v2.0/TitleBar.vue'
import { Window } from '@wailsio/runtime'

vi.mock('@wailsio/runtime', () => ({
  Window: {
    ToggleMaximise: vi.fn(),
    IsMaximised: vi.fn(() => Promise.resolve(false))
  }
}))

describe('TitleBar.vue', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('renders all sections and sub-components', () => {
    const wrapper = mount(TitleBar)
    console.log("HTML:", wrapper.html())
    
    // Header
    expect(wrapper.find('header.titlebar').exists()).toBe(true)
    
    // SearchBar check
    expect(wrapper.find('input[placeholder="Search requests..."]').exists()).toBe(true)
    
    // ActionMenu check
    expect(wrapper.find('button[aria-label="Add new item"]').exists()).toBe(true)
  })

  it('calls Wails ToggleMaximise on double-click', async () => {
    const wrapper = mount(TitleBar)
    await wrapper.find('header.titlebar').trigger('dblclick')
    expect(Window.ToggleMaximise).toHaveBeenCalled()
  })
})
