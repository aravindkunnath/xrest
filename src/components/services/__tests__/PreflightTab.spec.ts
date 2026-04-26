import { describe, it, expect, vi } from 'vitest';
import { mount } from '@vue/test-utils';
import PreflightTab from '../PreflightTab.vue';

import { createTestingPinia } from '@pinia/testing';

// Mock components that are not relevant to rendering tests
vi.mock('../../InterpolatedInput.vue', () => ({
  default: { 
    props: ['modelValue'],
    template: '<input :value="modelValue" />' 
  }
}));
vi.mock('../../InterpolatedTextarea.vue', () => ({
  default: { template: '<textarea></textarea>' }
}));
vi.mock('../../RequestParameters.vue', () => ({
  default: { template: '<div>Mock Params</div>' }
}));
vi.mock('@/components/TestPreflightDialog.vue', () => ({
  default: { template: '<div>Mock Dialog</div>' }
}));

describe('PreflightTab.vue', () => {
  const defaultPreflight = {
    enabled: true,
    method: 'POST',
    url: 'https://auth.example.com',
    body: '',
    bodyType: 'application/json',
    bodyParams: [],
    cacheToken: true,
    cacheDurationMode: 'derived',
    cacheDurationSeconds: 3600,
    cacheDurationKey: 'expires_in',
    cacheDurationUnit: 'seconds',
    tokenKey: 'access_token',
    tokenHeader: 'Authorization'
  };

  it('renders correctly when enabled', () => {
    const wrapper = mount(PreflightTab, {
      props: {
        preflight: defaultPreflight,
        variables: {},
        environmentName: 'test',
        serviceId: 'service-1'
      },
      global: {
        plugins: [createTestingPinia({ createSpy: vi.fn })]
      }
    });

    expect(wrapper.text()).toContain('Pre-flight Configuration');
    expect(wrapper.find('input[type="number"]').exists()).toBe(false); // Mode is 'derived'
  });

  it('shows manual duration input when mode is manual', async () => {
    const preflight = { ...defaultPreflight, cacheDurationMode: 'manual' };
    const wrapper = mount(PreflightTab, {
      props: {
        preflight,
        variables: {},
        environmentName: 'test',
        serviceId: 'service-1'
      },
      global: {
        plugins: [createTestingPinia({ createSpy: vi.fn })]
      }
    });

    expect(wrapper.find('input[type="number"]').exists()).toBe(true);
  });

  it('renders disabled state when preflight is disabled', () => {
    const wrapper = mount(PreflightTab, {
      props: {
        preflight: { ...defaultPreflight, enabled: false },
        variables: {},
        environmentName: 'test',
        serviceId: 'service-1'
      },
      global: {
        plugins: [createTestingPinia({ createSpy: vi.fn })]
      }
    });

    expect(wrapper.text()).toContain('Pre-flight is Disabled');
  });

  it('renders the preflight URL from props', () => {
    const url = 'https://api.example.com/oauth/token';
    const wrapper = mount(PreflightTab, {
      props: {
        preflight: { ...defaultPreflight, url },
        variables: {},
        environmentName: 'test',
        serviceId: 'service-1'
      },
      global: {
        plugins: [createTestingPinia({ createSpy: vi.fn })]
      }
    });

    // Check if the URL is passed correctly to the InterpolatedInput (which we mocked as <input />)
    // In our mock, v-model (modelValue) is what we should check
    const urlInput = wrapper.find('input');
    expect(urlInput.element.value).toBe(url);
  });
});
