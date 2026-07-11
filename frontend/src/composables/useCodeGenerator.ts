/**
 * Code Generator Composable
 *
 * Generates code snippets for API requests in various programming languages.
 * Supports curl, Node.js (Axios), Java (HttpClient), and Kotlin (OkHttp).
 */

/**
 * Generate a code snippet for the given language and tab configuration
 * @param language - Target language ('curl', 'node', 'java', 'kotlin')
 * @param tab - Tab object containing request configuration
 * @returns Generated code snippet as a string
 */
export const generateCodeSnippet = (language: string, tab: any): string => {
  if (!tab) return ''

  const method = tab.method
  const url = tab.url

  switch (language) {
    case 'curl':
      return `curl --request ${method} \\
  --url ${url} \\
  --header 'Content-Type: application/json' \\
  --data '${tab.body.content.replace(/'/g, "'\\''")}'`

    case 'node':
      return `const axios = require('axios');

axios({
  method: '${method.toLowerCase()}',
  url: '${url}',
  data: ${tab.body.content}
}).then(console.log);`

    case 'java':
      return `HttpRequest request = HttpRequest.newBuilder()
    .uri(URI.create("${url}"))
    .header("Content-Type", "application/json")
    .method("${method}", HttpRequest.BodyPublishers.ofString("${tab.body.content.replace(/"/g, '\\"').replace(/\n/g, '')}"))
    .build();`

    case 'kotlin':
      return `val client = OkHttpClient()
val mediaType = "application/json".toMediaType()
val body = "${tab.body.content.replace(/"/g, '\\"').replace(/\n/g, '')}".toRequestBody(mediaType)
val request = Request.Builder()
  .url("${url}")
  .post(body)
  .addHeader("Content-Type", "application/json")
  .build()`

    default:
      return ''
  }
}

/**
 * Hook to use the code generator
 * @returns Object with generateCodeSnippet function
 */
export const useCodeGenerator = () => {
  return {
    generateCodeSnippet
  }
}
