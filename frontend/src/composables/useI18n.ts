import { ref, computed } from 'vue'

const currentLocale = ref('en')

const translations: Record<string, Record<string, string>> = {
  en: {
    'sidebar.services': 'Services',
    'sidebar.environments': 'Environments',
    'sidebar.settings': 'Settings',
  },
  fr: {
    'sidebar.services': 'Services',
    'sidebar.environments': 'Environnements',
    'sidebar.settings': 'Paramètres',
  }
}

export function useI18n() {
  const t = (key: string): string => {
    return translations[currentLocale.value]?.[key] || key
  }

  const locale = computed({
    get: () => currentLocale.value,
    set: (val: string) => {
      if (translations[val]) {
        currentLocale.value = val
      }
    }
  })

  return {
    t,
    locale,
    availableLocales: Object.keys(translations)
  }
}
