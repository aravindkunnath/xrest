import { mount } from "@vue/test-utils";
import { describe, expect, it } from "vitest";
import RequestParameters from "@/features/request/components/RequestParameters.vue";
import Checkbox from "@/components/ui/checkbox/Checkbox.vue";

const mountComponent = (items: any[]) =>
    mount(RequestParameters, {
        props: {
            items,
            variables: {},
            environmentName: "",
        },
        global: {
            stubs: {
                InterpolatedInput: {
                    props: ["modelValue"],
                    template: "<input :value=\"modelValue\" />",
                },
            },
        },
    });

describe("RequestParameters bulk edit", () => {
    it("serializes active and inactive rows and restores them to the table", async () => {
        const items = [
            { enabled: true, name: "active", value: "one" },
            { enabled: false, name: "inactive", value: "two" },
        ];
        const wrapper = mountComponent(items);

        await wrapper.get("button").trigger("click");
        expect(wrapper.get("textarea").element.value).toBe(
            "active: one\n// inactive: two",
        );

        await wrapper.get("textarea").setValue("new: value\n// disabled: item");
        await wrapper.get("textarea").trigger("input");
        await wrapper.get("button").trigger("click");

        expect(items).toEqual([
            { enabled: true, name: "new", value: "value" },
            { enabled: false, name: "disabled", value: "item" },
        ]);
    });

    it("keeps bulk text synchronized with table changes while open", async () => {
        const wrapper = mountComponent([{ enabled: false, name: "old", value: "value" }]);

        await wrapper.get("button").trigger("click");
        await wrapper.setProps({
            items: [{ enabled: false, name: "changed", value: "value" }],
        });

        expect(wrapper.get("textarea").element.value).toBe("// changed: value");
    });

    it("enables a row when its bulk-edit comment is removed", async () => {
        const items = [{ enabled: false, name: "disabled", value: "item" }];
        const wrapper = mountComponent(items);

        await wrapper.get("button").trigger("click");
        await wrapper.get("textarea").setValue("disabled: item");
        await wrapper.get("button").trigger("click");

        expect(items).toEqual([
            { enabled: true, name: "disabled", value: "item" },
        ]);
    });

    it("preserves checkbox changes and reflects them in bulk edit", async () => {
        const items = [{ enabled: true, name: "key", value: "value" }];
        const wrapper = mountComponent(items);

        wrapper.findComponent(Checkbox).vm.$emit("update:checked", false);
        await wrapper.vm.$nextTick();
        expect(items[0].enabled).toBe(false);

        await wrapper.get("button").trigger("click");
        expect(wrapper.get("textarea").element.value).toBe("// key: value");
    });
});
