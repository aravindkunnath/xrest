import { mount } from "@vue/test-utils";
import { describe, expect, it } from "vitest";
import RequestAuth from "@/features/request/components/RequestAuth.vue";

const makeAuth = (type: string) => ({
    type,
    active: true,
    basicUser: "",
    basicPass: "",
    bearerToken: "",
    apiKeyName: "",
    apiKeyValue: "",
    apiKeyLocation: "header",
});

const preflight = {
    enabled: true,
    method: "POST",
    url: "",
    body: "",
    bodyType: "application/json",
    bodyParams: [],
    headers: [],
    cacheToken: true,
    cacheDuration: "derived",
    cacheDurationKey: "expires_in",
    cacheDurationUnit: "seconds",
    tokenKey: "access_token",
    tokenHeader: "Authorization",
};

const mountComponent = (authType: string) =>
    mount(RequestAuth, {
        props: {
            auth: makeAuth(authType),
            preflight,
            variables: {},
            environmentName: "DEV",
            serviceId: "s-1",
        },
        global: {
            stubs: {
                Select: { template: "<div><slot /></div>" },
                SelectTrigger: { template: "<button><slot /></button>" },
                SelectValue: { template: "<span />" },
                SelectContent: true,
                SelectItem: { template: "<div><slot /></div>" },
                Input: { template: "<input />" },
                Label: { template: "<label><slot /></label>" },
                Switch: {
                    props: ["modelValue"],
                    template:
                        '<button class="switch" @click="$emit(\'update:modelValue\', !modelValue)"></button>',
                },
                InterpolatedInput: { template: "<input />" },
                InterpolatedTextarea: { template: "<textarea />" },
                RequestParameters: { template: "<div />" },
                ShieldCheck: { template: "<span />" },
                Globe: { template: "<span />" },
                Play: { template: "<span />" },
            },
        },
    });

describe("RequestAuth — pre-flight gating", () => {
    it("shows pre-flight editor for bearer auth", async () => {
        const wrapper = mountComponent("bearer");
        expect(wrapper.text()).toContain("Authentication Pre-flight");
    });

    it("hides pre-flight editor for non-bearer auth", async () => {
        const wrapper = mountComponent("basic");
        expect(wrapper.text()).not.toContain("Authentication Pre-flight");
    });
});