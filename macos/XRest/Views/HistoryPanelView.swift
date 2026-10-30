import SwiftUI

/// History panel: filters, paged newest-first list, request/response detail.
struct HistoryPanelView: View {
    @Bindable var model: WorkspaceModel

    private static let methods = ["GET", "POST", "PUT", "PATCH", "DELETE", "HEAD", "OPTIONS"]

    var body: some View {
        VStack(spacing: 0) {
            HStack {
                Text("History").font(.ui(14, weight: .semibold))
                Spacer()
                Button("Refresh") { model.reloadHistory() }
                Button("Clear…") { model.requestClearHistory() }
                    .disabled(model.historyRows.isEmpty && model.historyError == nil)
                Button("Close") { model.closePanel() }
            }
            .padding(.horizontal, 16)
            .frame(height: 40)
            .hairline(.bottom)

            if let error = model.historyError {
                Text(error)
                    .font(.mono(11)).foregroundStyle(Theme.red)
                    .textSelection(.enabled)
                    .padding(8)
                    .frame(maxWidth: .infinity, alignment: .leading)
                    .chrome(fill: Theme.redFill, stroke: Theme.redBorder, radius: 6)
                    .padding(12)
            }
            filters
            HStack(spacing: 0) {
                list.frame(width: 440)
                    .hairline(.trailing)
                detail
            }
        }
        .onChange(of: model.historyFilter) { model.reloadHistory() }
        .confirmationDialog(
            "Clear all history?", isPresented: $model.pendingClearHistory, titleVisibility: .visible
        ) {
            Button("Clear History", role: .destructive) { model.confirmClearHistory() }
            Button("Cancel", role: .cancel) {}
        } message: {
            Text("This deletes the recorded requests and responses for every service. It cannot be undone.")
        }
    }

    private var filters: some View {
        HStack(spacing: 12) {
            Toggle("This service only", isOn: $model.historyFilter.thisServiceOnly)
            Picker("Endpoint", selection: $model.historyFilter.endpointId) {
                Text("Any endpoint").tag(String?.none)
                ForEach(model.endpoints, id: \.id) { Text($0.name).tag(Optional($0.id)) }
            }
            .frame(maxWidth: 220)
            Picker("Method", selection: $model.historyFilter.method) {
                Text("Any method").tag(String?.none)
                ForEach(Self.methods, id: \.self) { Text($0).tag(Optional($0)) }
            }
            .frame(maxWidth: 160)
            Picker("Status", selection: $model.historyFilter.statusClass) {
                Text("Any status").tag(UInt8?.none)
                Text("2xx").tag(Optional(UInt8(2)))
                Text("3xx").tag(Optional(UInt8(3)))
                Text("4xx").tag(Optional(UInt8(4)))
                Text("5xx").tag(Optional(UInt8(5)))
                Text("Failed").tag(Optional(UInt8(0)))
            }
            .frame(maxWidth: 150)
            Spacer()
        }
        .padding(.horizontal, 16)
        .padding(.vertical, 8)
        .hairline(.bottom)
    }

    private var list: some View {
        ScrollView {
            LazyVStack(spacing: 0) {
                if model.historyRows.isEmpty && model.historyError == nil {
                    Text("No history yet. Sent requests are recorded here.")
                        .font(.ui(12)).foregroundStyle(Theme.muted).padding(16)
                }
                ForEach(model.historyRows, id: \.id) { row in
                    Button { model.selectHistory(row.id) } label: { rowView(row) }
                        .buttonStyle(.plain)
                }
                if model.historyPaging.hasMore {
                    Button("Load more") { model.loadMoreHistory() }.padding(12)
                }
            }
        }
    }

    private func rowView(_ r: HistorySummary) -> some View {
        VStack(alignment: .leading, spacing: 3) {
            HStack(spacing: 8) {
                Text(r.method).font(.mono(11, weight: .semibold)).foregroundStyle(Theme.muted)
                Text(HistoryFormat.status(r))
                    .font(.mono(11, weight: .semibold))
                    .foregroundStyle(tone(r))
                Spacer()
                Text(HistoryFormat.age(createdAtMs: r.createdAt)).font(.ui(11)).foregroundStyle(Theme.muted)
            }
            Text(r.url).font(.mono(12)).lineLimit(1).truncationMode(.middle)
            HStack(spacing: 8) {
                Text(HistoryFormat.size(r))
                Text(HistoryFormat.duration(r))
                if let env = r.environment { Text(env) }
                Text(r.serviceName)
            }
            .font(.ui(11)).foregroundStyle(Theme.muted).lineLimit(1)
        }
        .padding(.horizontal, 16)
        .padding(.vertical, 8)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(model.historyDetail?.id == r.id ? Theme.selected : .clear)
        .hairline(.top, color: Theme.rowDivider)
        .contentShape(Rectangle())
    }

    private func tone(_ r: HistorySummary) -> Color {
        r.status.map(Theme.statusColor) ?? Theme.red
    }

    @ViewBuilder
    private var detail: some View {
        if let e = model.historyDetail {
            VStack(spacing: 0) {
                HStack {
                    Text("\(e.method) \(e.url)").font(.mono(12)).lineLimit(1).truncationMode(.middle)
                        .textSelection(.enabled)
                    Spacer()
                    Button("Open as scratch request") { model.openHistoryAsScratch() }
                }
                .padding(.horizontal, 16)
                .frame(height: 40)
                .hairline(.bottom)
                ScrollView {
                    VStack(alignment: .leading, spacing: 6) {
                        Text("REQUEST HEADERS").font(.ui(11, weight: .semibold)).foregroundStyle(Theme.muted)
                        pairs(e.requestHeaders)
                        if !e.requestBody.isEmpty {
                            Text("REQUEST BODY").font(.ui(11, weight: .semibold)).foregroundStyle(Theme.muted)
                                .padding(.top, 8)
                            Text(e.requestBody).font(.mono(12)).textSelection(.enabled)
                        }
                    }
                    .padding(16)
                    .frame(maxWidth: .infinity, alignment: .leading)
                }
                .frame(maxHeight: 180)
                .hairline(.bottom)
                if let response = e.asResponse {
                    if e.truncated {
                        Text("Body was cut at 1 MiB when recorded.")
                            .font(.ui(12)).foregroundStyle(Theme.amber).padding(6)
                    }
                    ResponseContent(response: response).id(e.id)
                } else {
                    responsePlaceholder(
                        "Request failed", detail: e.error ?? "No response was received.",
                        symbol: "exclamationmark.octagon", tint: Theme.red)
                }
            }
        } else {
            responsePlaceholder(
                "Select an entry", detail: "Pick a request to see what was sent and received.",
                symbol: "clock", tint: Theme.muted)
        }
    }

    private func pairs(_ headers: [HeaderPair]) -> some View {
        VStack(alignment: .leading, spacing: 2) {
            if headers.isEmpty { Text("None").font(.ui(12)).foregroundStyle(Theme.muted) }
            ForEach(headers.indices, id: \.self) { i in
                HStack(alignment: .top, spacing: 12) {
                    Text(headers[i].name).foregroundStyle(Theme.blue).frame(width: 160, alignment: .leading)
                    Text(headers[i].value).foregroundStyle(Theme.textSoft).textSelection(.enabled)
                }
                .font(.mono(12))
            }
        }
    }
}
