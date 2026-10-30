import { describe, it, expect, beforeEach, vi } from "vitest";
import { mount } from "@vue/test-utils";
import { nextTick } from "vue";
import ServiceSidebar from "@/features/services/components/ServiceSidebar.vue";
import { createPinia, setActivePinia } from "pinia";
import { useServicesStore } from "@/stores/services";

vi.mock("@/composables/useGitIntegration", () => ({
    useGitIntegration: () => ({
        gitStatuses: {
            value: {
                "service-1": {
                    isGit: true,
                    branch: "main",
                    hasUncommittedChanges: true,
                    hasUnpushedCommits: false,
                    uncommittedFiles: [
                        { path: "endpoints/user.yaml", status: "modified" },
                        { path: "endpoints/legacy.yaml", status: "deleted" },
                    ],
                },
                "service-2": {
                    isGit: false,
                },
            },
        },
        handlePullGit: vi.fn(),
        handlePushGit: vi.fn(),
        handleInitGit: vi.fn(),
        handleCommitGit: vi.fn(),
        fetchGitStatus: vi.fn(),
    }),
}));

describe("ServiceSidebar.vue", () => {
    let pinia: ReturnType<typeof createPinia>;

    const mockServices = [
        {
            id: "service-1",
            name: "User Microservice",
            directory: "/services/user",
            isAuthenticated: false,
            endpoints: [
                {
                    id: "ep-1",
                    serviceId: "service-1",
                    name: "Get User",
                    method: "GET",
                    url: "/users/{id}",
                    metadata: { version: "1.0", lastUpdated: Date.now() },
                    lastVersion: 1,
                },
                {
                    id: "ep-2",
                    serviceId: "service-1",
                    name: "Create User",
                    method: "POST",
                    url: "/users",
                    metadata: { version: "2.0", lastUpdated: Date.now() },
                    lastVersion: 2,
                },
            ],
            environments: [{ name: "DEV", isUnsafe: false, variables: [] }],
        },
        {
            id: "service-2",
            name: "Payment Gateway",
            directory: "/services/payment",
            isAuthenticated: false,
            endpoints: [],
            environments: [],
        },
    ];

    beforeEach(() => {
        pinia = createPinia();
        setActivePinia(pinia);

        const store = useServicesStore();
        store.services = mockServices as any;
    });

    it("should render the active service's endpoints based on the store selection", async () => {
        const store = useServicesStore();
        store.selectedServiceId = "service-1";

        const wrapper = mount(ServiceSidebar, {
            global: {
                plugins: [pinia],
            },
        });

        expect(wrapper.text()).toContain("Get User");
        expect(wrapper.text()).toContain("Create User");

        store.setSelectedService("service-2");
        await nextTick();

        expect(wrapper.text()).not.toContain("Get User");
        expect(wrapper.text()).toContain("No endpoints found");
    });

    it("should display endpoints with HTTP method tags and version badges on the right", () => {
        const wrapper = mount(ServiceSidebar, {
            global: {
                plugins: [pinia],
            },
        });

        const endpoints = wrapper.findAll(".flex.items-center.justify-between.px-2.py-1\\.5");
        expect(endpoints.length).toBe(2);

        // Endpoint 1: GET, Get User, 1
        expect(endpoints[0].text()).toContain("GET");
        expect(endpoints[0].text()).toContain("Get User");
        expect(endpoints[0].text()).toContain("1");

        // Endpoint 2: POST, Create User, 2
        expect(endpoints[1].text()).toContain("POST");
        expect(endpoints[1].text()).toContain("Create User");
        expect(endpoints[1].text()).toContain("2");
    });

    it("should emit select-endpoint event when an endpoint is clicked", async () => {
        const wrapper = mount(ServiceSidebar, {
            global: {
                plugins: [pinia],
            },
        });

        const endpointRow = wrapper.find(".group");
        await endpointRow.trigger("click");

        expect(wrapper.emitted("select-endpoint")).toBeTruthy();
        expect(wrapper.emitted("select-endpoint")![0][0]).toEqual(mockServices[0].endpoints[0]);
    });

    it("should render Git integration section with branch and pull/push controls for Git repo", () => {
        const wrapper = mount(ServiceSidebar, {
            global: {
                plugins: [pinia],
            },
        });

        expect(wrapper.text()).toContain("Git");
        expect(wrapper.text()).toContain("main");
        expect(wrapper.text()).toContain("Uncommitted Changes");
    });

    it("should render Git Init button when active service has no Git repo", async () => {
        const store = useServicesStore();
        store.setSelectedService("service-2");

        const wrapper = mount(ServiceSidebar, {
            global: {
                plugins: [pinia],
            },
        });

        expect(wrapper.text()).toContain("Init");
        expect(wrapper.text()).toContain("No Git repository associated with this service directory.");
    });

    it("should emit add-endpoint event when add endpoint button is clicked", async () => {
        const wrapper = mount(ServiceSidebar, {
            global: {
                plugins: [pinia],
            },
        });

        const addEndpointBtn = wrapper.find('button[title="Add Endpoint"]');
        expect(addEndpointBtn.exists()).toBe(true);

        await addEndpointBtn.trigger("click");
        expect(wrapper.emitted("add-endpoint")).toBeTruthy();
    });
});
