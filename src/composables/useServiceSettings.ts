
import { useGitIntegration } from "@/composables/useGitIntegration";
import { useTabManager } from "@/composables/useTabManager";
import { useServicesStore } from "@/stores/services";
import { ask } from "@tauri-apps/plugin-dialog";
import { toast } from "vue-sonner";

/**
 * Composable to handle service and collection settings logic
 */
export function useServiceSettings() {
    const servicesStore = useServicesStore();
    const { closeTab, tabs, updateTabSnapshot } = useTabManager();
    const { handleSyncGit, handleInitGit } = useGitIntegration();

    /**
     * Save settings for a service or collection
     */
    const saveSettings = async (tab: any) => {
        try {
            const index = servicesStore.services.findIndex((s) => s.id === tab.serviceId);
            if (index !== -1) {
                console.info("Saving service", tab.serviceData);
                await servicesStore.updateService(
                    index,
                    tab.serviceData,
                    `Update settings for service: ${tab.serviceData.name}`
                );
                tab.title = tab.serviceData.name;
                updateTabSnapshot(tab);
                toast.success("Service updated successfully");
            }
        } catch (error) {
            toast.error(`Failed to update service`);
        }
    };

    /**
     * Delete a service or collection
     */
    const deleteItem = async (serviceId: string) => {
        const items = servicesStore.services;
        const index = items.findIndex((i: any) => i.id === serviceId);

        if (index !== -1) {
            const itemName = items[index].name;
            const confirmation = await ask(
                `Are you sure you want to delete service "${itemName}"? This will remove it from your workspace settings.`,
                { title: "Confirm Deletion", kind: "warning" }
            );

            if (confirmation) {
                try {
                    await servicesStore.deleteService(index);

                    // Close all tabs related to this service/collection
                    const tabsToClose = tabs.value
                        .filter((t: any) => t.serviceId === serviceId)
                        .map((t: any) => t.id);

                    tabsToClose.forEach(tid => closeTab(tid));

                    toast.success(`Service removed from workspace`);
                } catch (error) {
                    toast.error(`Failed to delete service`);
                }
            }
        }
    };

    /**
     * Reload all services and collections from disk
     */
    const reloadAll = async () => {
        console.info("Reloading services");
        try {
            await servicesStore.loadServices();
            toast.success("Settings reloaded from disk");
        } catch (error) {
            toast.error("Failed to reload items");
        }
    };

    return {
        saveSettings,
        deleteItem,
        reloadAll,
        syncGit: handleSyncGit,
        initGit: handleInitGit
    };
}
