import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount } from '@vue/test-utils'
import AddCollectionDialog from '../AddCollectionDialog.vue'
import { toast } from 'vue-sonner'

// Mock vue-sonner
vi.mock('vue-sonner', () => ({
    toast: {
        success: vi.fn(),
    },
}))

describe('AddCollectionDialog', () => {
    beforeEach(() => {
        vi.clearAllMocks()
    })

    // Helper to mount with necessary stubs if needed
    const mountDialog = (props = { open: true }) => {
        return mount(AddCollectionDialog, {
            props,
            global: {
                stubs: {
                    // Stub portal to render content in-place for easier testing
                    DialogPortal: {
                        template: '<div><slot /></div>'
                    },
                    // Also stub transitions to avoid timing issues
                    'transition': false
                }
            }
        })
    }

    it('renders when open is true', async () => {
        const wrapper = mountDialog({ open: true })
        // We might need to wait a tick for internal transitions
        await wrapper.vm.$nextTick()

        expect(wrapper.find('input').exists()).toBe(true)
        expect(wrapper.text()).toContain('New Collection')
    })

    it('emits collection-created and resets form when valid name is submitted', async () => {
        const wrapper = mountDialog({ open: true })
        await wrapper.vm.$nextTick()

        const input = wrapper.find('input')
        await input.setValue('Test Collection')

        const buttons = wrapper.findAll('button')
        const createButton = buttons.find(b => b.text().includes('Create Collection'))

        expect(createButton).toBeDefined()
        await createButton?.trigger('click')

        // Check emitted events
        const createdEvents = wrapper.emitted('collection-created')
        expect(createdEvents).toBeDefined()
        expect(createdEvents?.[0][0]).toMatchObject({
            name: 'Test Collection',
            isAuthenticated: false,
            authType: 'none',
        })

        const openEvents = wrapper.emitted('update:open')
        expect(openEvents).toBeDefined()
        expect(openEvents?.[0][0]).toBe(false)

        // Check toast
        expect(toast.success).toHaveBeenCalledWith('Collection Created', expect.any(Object))

        // Check form reset (input should be empty)
        expect((input.element as HTMLInputElement).value).toBe('')
    })

    it('disables create button when name is empty', async () => {
        const wrapper = mountDialog({ open: true })
        await wrapper.vm.$nextTick()

        const buttons = wrapper.findAll('button')
        const createButton = buttons.find(b => b.text().includes('Create Collection'))

        expect(createButton?.attributes('disabled')).toBeDefined()
    })

    it('emits update:open false when cancel is clicked', async () => {
        const wrapper = mountDialog({ open: true })
        await wrapper.vm.$nextTick()

        const cancelButton = wrapper.findAll('button').find(b => b.text().includes('Cancel'))
        await cancelButton?.trigger('click')

        expect(wrapper.emitted('update:open')?.[0][0]).toBe(false)
    })

    it('does not emit collection-created if name is only whitespace', async () => {
        const wrapper = mountDialog({ open: true })
        await wrapper.vm.$nextTick()

        const input = wrapper.find('input')
        await input.setValue('   ')

        const buttons = wrapper.findAll('button')
        const createButton = buttons.find(b => b.text().includes('Create Collection'))

        // The button might still be enabled because it checks name.length, 
        // but the handler has a trim() check.
        await createButton?.trigger('click')

        expect(wrapper.emitted('collection-created')).toBeUndefined()
    })

    it('closes dialog on enter key in input', async () => {
        const wrapper = mountDialog({ open: true })
        await wrapper.vm.$nextTick()

        const input = wrapper.find('input')
        await input.setValue('Another Collection')
        await input.trigger('keyup.enter')

        expect(wrapper.emitted('collection-created')).toBeDefined()
        expect(wrapper.emitted('update:open')?.[0][0]).toBe(false)
    })
})
