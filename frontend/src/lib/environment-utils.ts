/**
 * Environment Variable Management Utilities
 *
 * Pure functions for managing environment variables across different environments.
 * These utilities help maintain consistency when variables are added, renamed, or removed.
 */

export interface Variable {
  name: string
  value: string
  enabled: boolean
  secretKey?: string
}

export interface EnvironmentConfig {
  name: string
  isUnsafe: boolean
  variables: Variable[]
}

/**
 * Get unique variable names across all environments
 * @param environments - Array of environment configurations
 * @returns Array of unique variable names
 */
export const getUniqueVariableNames = (environments: EnvironmentConfig[]): string[] => {
  const names = new Set<string>()
  environments.forEach(env => {
    if (env.variables) {
      env.variables.forEach((v: Variable) => names.add(v.name))
    }
  })
  return Array.from(names)
}

/**
 * Sync a variable value for a specific environment
 * Creates the variable if it doesn't exist, updates if it does
 * @param env - Environment configuration to update
 * @param varName - Name of the variable
 * @param newValue - New value to set
 */
export const syncVariableValue = (env: EnvironmentConfig, varName: string, newValue: string): void => {
  if (!env.variables) env.variables = []
  const v = env.variables.find((v: Variable) => v.name === varName)
  if (v) {
    v.value = newValue
  } else {
    env.variables.push({ name: varName, value: newValue, enabled: true })
  }
}

/**
 * Sync a variable secret key for a specific environment
 * @param env - Environment configuration to update
 * @param varName - Name of the variable
 * @param secretKey - Secret key to link (or undefined to unlink)
 */
export const syncVariableSecret = (env: EnvironmentConfig, varName: string, secretKey: string | undefined): void => {
  if (!env.variables) env.variables = []
  const v = env.variables.find((v: Variable) => v.name === varName)
  if (v) {
    v.secretKey = secretKey
    // If linking a secret, optionally clear the value or keep it as placeholder?
    // Let's keep value as is for now, maybe UI clears it.
  } else {
    env.variables.push({ name: varName, value: '', enabled: true, secretKey })
  }
}

/**
 * Rename a variable across all environments
 * @param environments - Array of environment configurations
 * @param oldName - Current variable name
 * @param newName - New variable name
 */
export const syncVariableName = (environments: EnvironmentConfig[], oldName: string, newName: string): void => {
  environments.forEach(env => {
    if (!env.variables) return
    const v = env.variables.find((v: Variable) => v.name === oldName)
    if (v) v.name = newName
  })
}

/**
 * Remove a variable from all environments
 * @param environments - Array of environment configurations
 * @param varName - Name of the variable to remove
 */
export const removeVariable = (environments: EnvironmentConfig[], varName: string): void => {
  environments.forEach(env => {
    if (!env.variables) return
    const idx = env.variables.findIndex((v: Variable) => v.name === varName)
    if (idx !== -1) env.variables.splice(idx, 1)
  })
}

/**
 * Add a new variable to all environments with empty values
 * @param environments - Array of environment configurations
 */
export const addVariableToAll = (environments: EnvironmentConfig[]): void => {
  const name = `NEW_VAR_${Date.now()}`
  environments.forEach(env => {
    if (!env.variables) env.variables = []
    env.variables.push({ name, value: '', enabled: true })
  })
}
