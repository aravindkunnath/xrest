import { describe, it, expect } from 'vitest'
import { mount } from '@vue/test-utils'
import SearchBar from '@/components/v2.0/SearchBar.vue'

describe('SearchBar.vue', () => {
  it('updates the modelValue when input is typed', async () => {
    const wrapper = mount(SearchBar, {
      props: {
        modelValue: '',
        'onUpdate:modelValue': (e: string) => wrapper.setProps({ modelValue: e })
      }
    })

    const input = wrapper.find('input')
    await input.setValue('test search')
    expect(wrapper.props('modelValue')).toBe('test search')
  })
})
