import { computed } from 'vue';
import { useServicesStore } from '@/stores/services';

export const useActiveVariables = (serviceId?: string) => {
    const servicesStore = useServicesStore();

    const variables = computed(() => {
        if (!serviceId) return {};

        const service = servicesStore.services.find((s) => s.id === serviceId);
        if (!service) return {};

        const envName = service.selectedEnvironment || (service.environments[0]?.name);
        const env = service.environments.find((e) => e.name === envName);

        if (!env) return {};

        const vars: Record<string, string> = {};
        env.variables.forEach((v) => {
            vars[v.name] = v.value;
        });

        return vars;
    });

    return variables;
};
