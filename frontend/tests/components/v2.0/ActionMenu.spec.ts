import { describe, it, expect, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { createTestingPinia } from '@pinia/testing'
import ActionMenu from '@/components/v2.0/ActionMenu.vue'

vi.mock('vue-router', () => ({
  useRouter: () => ({
    push: vi.fn(),
  }),
}))

describe('ActionMenu.vue', () => {
  it('toggles dropdown visibility when button is clicked', async () => {
    const wrapper = mount(ActionMenu, {
      global: {
        plugins: [createTestingPinia()],
      },
    })
    expect(wrapper.find('[role="menu"]').exists()).toBe(false)

    // Click trigger button
    await wrapper.find('button[aria-label="Add new item"]').trigger('click')
    expect(wrapper.find('[role="menu"]').exists()).toBe(true)

    // Click trigger button again
    await wrapper.find('button[aria-label="Add new item"]').trigger('click')
    expect(wrapper.find('[role="menu"]').exists()).toBe(false)
  })

  it('closes dropdown when clicking an action', async () => {
    const wrapper = mount(ActionMenu, {
      global: {
        plugins: [createTestingPinia()],
      },
    })
    
    // Open dropdown
    await wrapper.find('button[aria-label="Add new item"]').trigger('click')
    expect(wrapper.find('[role="menu"]').exists()).toBe(true)

    // Click first action button
    const actions = wrapper.findAll('button[role="menuitem"]')
    expect(actions.length).toBeGreaterThan(0)
    await actions[0].trigger('click')

    expect(wrapper.find('[role="menu"]').exists()).toBe(false)
  })
})

