import en from './en'
import fr from './fr'

function flattenKeys(obj: any, prefix = ''): string[] {
  return Object.keys(obj).reduce((keys: string[], key: string) => {
    const value = obj[key]
    const newKey = prefix ? `${prefix}.${key}` : key

    if (typeof value === 'object' && !Array.isArray(value) && value !== null) {
      keys.push(...flattenKeys(value, newKey))
    } else {
      keys.push(newKey)
    }

    return keys
  }, [])
}

function validateTranslations() {
  // Validate that all keys exist in both languages
  const enKeys = new Set(flattenKeys(en))
  const frKeys = new Set(flattenKeys(fr))

  const missingInFr = [...enKeys].filter(key => !frKeys.has(key))
  const missingInEn = [...frKeys].filter(key => !enKeys.has(key))

  if (missingInFr.length > 0) {
    throw new Error('Keys missing in French translation: ' + missingInFr.join(', '))
  }

  if (missingInEn.length > 0) {
    throw new Error('Keys missing in English translation: ' + missingInEn.join(', '))
  }
}

export default defineI18nConfig(() => {
  validateTranslations();
  const commonDateTimeFormat = {
    short: {
      year: 'numeric', month: 'short', day: 'numeric'
    },
    long: {
      year: 'numeric', month: 'short', day: 'numeric',
      weekday: 'short', hour: 'numeric', minute: 'numeric'
    },
    verbose: {
      year: 'numeric', month: 'short', day: 'numeric',
      weekday: 'long', hour: 'numeric', minute: 'numeric', second: 'numeric'
    },
    log: {
      year: 'numeric', month: '2-digit', day: '2-digit',
      hour: '2-digit', minute: '2-digit', second: '2-digit', hour12: false
    }
  };

  return {
    legacy: false,
    locale: 'en',
    messages: {
      en,
      fr
    },
    datetimeFormats: {
      'fr': commonDateTimeFormat,
      'en': commonDateTimeFormat

    }
  }
});
