export type Token = {
    type: 'text' | 'variable';
    content: string;
    resolvedValue?: string;
    isValid?: boolean;
    isSecret?: boolean;
};

const MAX_RECURSION_DEPTH = 10;

/**
 * Resolves variables in a string recursively.
 */
export const resolveVariables = (
    text: string,
    variables: Record<string, string> = {},
    depth = 0
): string => {
    if (depth >= MAX_RECURSION_DEPTH) return text;

    const replaced = text.replace(/\{\{(.*?)\}\}/g, (match, varName) => {
        const trimmedName = varName.trim();
        const value = variables[trimmedName];
        return value !== undefined ? value : match;
    });

    // If the replaced text still contains variables and changed from previous, recurse
    if (replaced !== text && replaced.includes('{{')) {
        return resolveVariables(replaced, variables, depth + 1);
    }

    return replaced;
};

export const parseInterpolation = (
    text: string,
    variables: Record<string, string> = {},
    secrets: string[] = []
): Token[] => {
    const tokens: Token[] = [];
    const regex = /\{\{(.*?)\}\}/g;
    let lastIndex = 0;
    let match;

    while ((match = regex.exec(text)) !== null) {
        if (match.index > lastIndex) {
            tokens.push({
                type: 'text',
                content: text.substring(lastIndex, match.index),
            });
        }

        const fullMatch = match[0];
        const varName = match[1].trim();

        const isSecret = varName.startsWith('secret.');
        let isValid = false;
        let resolvedValue: string | undefined = undefined;

        if (isSecret) {
            const secretKey = varName.substring(7); // "secret." is 7 chars
            isValid = secrets.includes(secretKey);
            resolvedValue = isValid ? '********' : undefined;
        } else {
            isValid = variables[varName] !== undefined;
            if (isValid) {
                resolvedValue = resolveVariables(variables[varName], variables);
            }
        }

        tokens.push({
            type: 'variable',
            content: fullMatch,
            resolvedValue,
            isValid,
            isSecret,
        });

        lastIndex = regex.lastIndex;
    }

    if (lastIndex < text.length) {
        tokens.push({
            type: 'text',
            content: text.substring(lastIndex),
        });
    }

    if (tokens.length === 0 && text.length > 0) {
        tokens.push({
            type: 'text',
            content: text,
        });
    }

    return tokens;
};
