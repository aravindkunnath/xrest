import Foundation

/// The one execution-history database per app process, shared by every workspace window.
/// A failed open is kept as `error` so the app still works and the History panel can say why.
final class HistoryStore: @unchecked Sendable {
    let history: XRestHistory?
    let error: String?

    static var defaultURL: URL {
        let base =
            FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask).first
            ?? FileManager.default.homeDirectoryForCurrentUser.appendingPathComponent("Library/Application Support")
        return base.appendingPathComponent("XRest/history.sqlite")
    }

    /// Opened lazily on first use.
    static let shared = HistoryStore(path: defaultURL)

    init(path: URL) {
        do {
            try FileManager.default.createDirectory(
                at: path.deletingLastPathComponent(), withIntermediateDirectories: true)
            history = try XRestHistory.open(path: path.path)
            error = nil
        } catch {
            history = nil
            self.error = "Could not open the history database: \(describe(error))"
        }
    }
}

/// Filter controls of the History panel.
struct HistoryFilterState: Equatable {
    /// Preselected: only the open service. Off shows every service.
    var thisServiceOnly = true
    var endpointId: String?
    var method: String?
    /// 2...5 for that class, 0 for failed requests.
    var statusClass: UInt8?

    func filter(serviceId: String) -> HistoryFilter {
        HistoryFilter(
            serviceId: thisServiceOnly ? serviceId : nil, endpointId: endpointId, method: method,
            statusClass: statusClass)
    }
}

/// Load-more paging state.
struct HistoryPaging: Equatable {
    var pageSize = 50
    private(set) var loaded = 0
    private(set) var total = 0

    init(pageSize: Int = 50) { self.pageSize = pageSize }

    var nextOffset: Int { loaded }
    var hasMore: Bool { loaded < total }

    mutating func reset(total: Int) {
        loaded = 0
        self.total = total
    }

    mutating func didLoad(count: Int) { loaded += count }
}

/// Row text for the list.
enum HistoryFormat {
    static func status(_ s: HistorySummary) -> String {
        s.status.map { String($0) } ?? "Failed"
    }

    static func tone(_ s: HistorySummary) -> StatusTone {
        s.status.map(StatusTone.of) ?? .error
    }

    static func size(_ s: HistorySummary) -> String {
        s.sizeBytes == 0 ? "0 B" : ByteCountFormatter.string(fromByteCount: Int64(s.sizeBytes), countStyle: .file)
    }

    static func duration(_ s: HistorySummary) -> String {
        s.timeMs < 1000 ? "\(s.timeMs) ms" : String(format: "%.1f s", Double(s.timeMs) / 1000)
    }

    static func age(createdAtMs: Int64, now: Date = Date()) -> String {
        let seconds = Int(now.timeIntervalSince1970 - Double(createdAtMs) / 1000)
        switch seconds {
        case ..<60: return "just now"
        case ..<3600: return "\(seconds / 60) min ago"
        case ..<86_400: return "\(seconds / 3600) h ago"
        default: return "\(seconds / 86_400) d ago"
        }
    }
}

/// History entry to scratch request conversion.
enum HistoryScratch {
    static let mask = "••••••"

    struct Result {
        var name: String
        var spec: RequestSpec
        var notice: String?
    }

    static func make(from e: HistoryEntry) -> Result {
        var notes: [String] = []
        let dropped = e.requestHeaders.filter { $0.value.contains(mask) }
        let kept = e.requestHeaders.filter { !$0.value.contains(mask) }
        if !dropped.isEmpty {
            notes.append("Masked headers were left out: " + dropped.map(\.name).joined(separator: ", ") + ".")
        }
        let isMultipart = e.requestHeaders.contains {
            $0.name.lowercased() == "content-type" && $0.value.lowercased().hasPrefix("multipart/")
        }
        var body = RequestBody.empty
        if e.requestBody.hasPrefix("<binary body") || isMultipart {
            notes.append("The request body was binary or multipart and was not copied; add it again.")
        } else if !e.requestBody.isEmpty {
            body = .raw(content: e.requestBody, contentType: nil)
        }
        let spec = RequestSpec(
            method: e.method, url: e.url, authenticated: false, params: [], pathParams: [],
            headers: kept.map { NameValue(name: $0.name, value: $0.value, enabled: true, secretKey: nil, kind: "plain") },
            body: body, options: RequestDraft.defaultOptions, preflight: nil)
        return Result(
            name: "\(e.method) \(e.url)".prefix(60).description, spec: spec,
            notice: notes.isEmpty ? nil : notes.joined(separator: " "))
    }
}

extension HistoryEntry {
    /// The stored response in the shape the response viewer takes; nil for failed requests.
    var asResponse: HttpResponse? {
        guard let status else { return nil }
        let type = responseHeaders.first { $0.name.lowercased() == "content-type" }?.value ?? ""
        return HttpResponse(
            status: status, statusText: statusText, headers: responseHeaders, contentType: type,
            body: responseBody, bodyIsBase64: bodyIsBase64, prettyBody: nil, sizeBytes: sizeBytes,
            timeMs: timeMs, finalUrl: url, environment: environment, unresolvedVariables: [],
            cookies: [])
    }
}
