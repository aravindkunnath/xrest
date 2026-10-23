import SwiftUI
import WebKit

/// Renders untrusted response HTML with JavaScript off, no base URL, and every navigation or
/// subresource load blocked by a content rule list. Fails closed: the HTML is only handed to
/// WebKit after the block-all rule list is installed; if it cannot be compiled the source is
/// shown as plain text with a notice.
struct HTMLPreview: View {
    let html: String

    @State private var state = RuleListState.pending
    @State private var ruleList: WKContentRuleList?

    var body: some View {
        switch HTMLPreviewPolicy.decision(for: state) {
        case .wait:
            Color.clear.task { await compile() }
        case .loadHTML:
            if let ruleList {
                HTMLWebView(html: html, ruleList: ruleList)
            }
        case .showSource:
            VStack(alignment: .leading, spacing: 6) {
                Text("Preview unavailable: the content blocker could not be installed. Showing source.")
                    .font(.ui(11)).foregroundStyle(Theme.amber)
                ScrollView {
                    Text(html).font(.mono(12)).textSelection(.enabled)
                        .frame(maxWidth: .infinity, alignment: .leading)
                }
            }
            .padding(10)
        }
    }

    private func compile() async {
        let rules = #"[{"trigger":{"url-filter":".*"},"action":{"type":"block"}}]"#
        let list = try? await WKContentRuleListStore.default().compileContentRuleList(
            forIdentifier: "xrest-block-all", encodedContentRuleList: rules)
        ruleList = list
        state = list == nil ? .failed : .ready
    }
}

private struct HTMLWebView: NSViewRepresentable {
    let html: String
    let ruleList: WKContentRuleList

    func makeNSView(context: Context) -> WKWebView {
        let config = WKWebViewConfiguration()
        config.defaultWebpagePreferences.allowsContentJavaScript = false
        config.preferences.javaScriptCanOpenWindowsAutomatically = false
        // Installed before any HTML is loaded.
        config.userContentController.add(ruleList)
        let web = WKWebView(frame: .zero, configuration: config)
        web.navigationDelegate = context.coordinator
        web.setValue(false, forKey: "drawsBackground")
        context.coordinator.load(html, in: web)
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

enum RuleListState: Equatable { case pending, ready, failed }

/// Fail-closed decision for the HTML preview: HTML is only loaded once the block-all content
/// rule list is installed; if it cannot be compiled the source is shown as text.
enum HTMLPreviewPolicy {
    enum Decision: Equatable { case wait, loadHTML, showSource }

    static func decision(for state: RuleListState) -> Decision {
        switch state {
        case .pending: return .wait
        case .ready: return .loadHTML
        case .failed: return .showSource
        }
    }
}
