import SwiftUI
import WebKit

/// Renders untrusted response HTML with JavaScript off, no base URL, and every navigation or
/// subresource load blocked by a content rule list.
struct HTMLPreview: NSViewRepresentable {
    let html: String

    func makeNSView(context: Context) -> WKWebView {
        let config = WKWebViewConfiguration()
        config.defaultWebpagePreferences.allowsContentJavaScript = false
        config.preferences.javaScriptCanOpenWindowsAutomatically = false
        let web = WKWebView(frame: .zero, configuration: config)
        web.navigationDelegate = context.coordinator
        web.setValue(false, forKey: "drawsBackground")
        // Block all network subresources (images, CSS, frames...) of the preview.
        let rules = #"[{"trigger":{"url-filter":".*"},"action":{"type":"block"}}]"#
        WKContentRuleListStore.default().compileContentRuleList(
            forIdentifier: "xrest-block-all", encodedContentRuleList: rules
        ) { list, _ in
            if let list {
                web.configuration.userContentController.add(list)
            }
            context.coordinator.load(html, in: web)
        }
        return web
    }

    func updateNSView(_ web: WKWebView, context: Context) {
        if let loaded = context.coordinator.loaded, loaded != html { context.coordinator.load(html, in: web) }
    }

    func makeCoordinator() -> Coordinator { Coordinator() }

    final class Coordinator: NSObject, WKNavigationDelegate {
        var loaded: String?

        func load(_ html: String, in web: WKWebView) {
            loaded = html
            web.loadHTMLString(html, baseURL: nil)
        }

        func webView(
            _ webView: WKWebView, decidePolicyFor action: WKNavigationAction,
            decisionHandler: @escaping (WKNavigationActionPolicy) -> Void
        ) {
            // Only the initial loadHTMLString (about:blank) may proceed.
            let isInitial = action.navigationType == .other && action.request.url?.scheme == "about"
            decisionHandler(isInitial ? .allow : .cancel)
        }
    }
}
