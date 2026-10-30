import XCTest

@testable import XRest

final class HistorySupportTests: XCTestCase {
    private func summary(
        id: Int64 = 1, status: UInt16? = 200, error: String? = nil, size: UInt64 = 2048, time: UInt64 = 12
    ) -> HistorySummary {
        HistorySummary(
            id: id, createdAt: 1_700_000_000_000, serviceId: "s", serviceName: "svc", endpointId: nil,
            environment: "dev", method: "GET", url: "https://h/x", status: status, truncated: false,
            timeMs: time, sizeBytes: size, error: error)
    }

    private func entry(
        headers: [HeaderPair] = [], body: String = ""
    ) -> HistoryEntry {
        HistoryEntry(
            id: 1, createdAt: 0, serviceId: "s", serviceName: "svc", endpointId: nil, environment: nil,
            method: "POST", url: "https://h/x", requestHeaders: headers, requestBody: body, status: 200,
            statusText: "OK", responseHeaders: [], responseBody: "", bodyIsBase64: false, truncated: false,
            timeMs: 1, sizeBytes: 0, error: nil)
    }

    func testFilterThisServiceByDefault() {
        let state = HistoryFilterState()
        let f = state.filter(serviceId: "svc-1")
        XCTAssertEqual(f.serviceId, "svc-1")
        XCTAssertNil(f.method)
        XCTAssertNil(f.statusClass)
        XCTAssertNil(f.endpointId)
    }

    func testFilterAllServicesAndFields() {
        var state = HistoryFilterState()
        state.thisServiceOnly = false
        state.method = "POST"
        state.statusClass = 4
        state.endpointId = "e1"
        let f = state.filter(serviceId: "svc-1")
        XCTAssertNil(f.serviceId)
        XCTAssertEqual(f.method, "POST")
        XCTAssertEqual(f.statusClass, 4)
        XCTAssertEqual(f.endpointId, "e1")
    }

    func testPaging() {
        var p = HistoryPaging(pageSize: 2)
        XCTAssertEqual(p.nextOffset, 0)
        p.reset(total: 5)
        XCTAssertTrue(p.hasMore)
        p.didLoad(count: 2)
        XCTAssertEqual(p.nextOffset, 2)
        XCTAssertTrue(p.hasMore)
        p.didLoad(count: 2)
        p.didLoad(count: 1)
        XCTAssertFalse(p.hasMore)
        p.reset(total: 0)
        XCTAssertFalse(p.hasMore)
        XCTAssertEqual(p.nextOffset, 0)
    }

    func testRowFormatting() {
        XCTAssertEqual(HistoryFormat.status(summary(status: 404)), "404")
        XCTAssertEqual(HistoryFormat.status(summary(status: nil, error: "boom")), "Failed")
        XCTAssertEqual(HistoryFormat.tone(summary(status: 204)), .success)
        XCTAssertEqual(HistoryFormat.tone(summary(status: 302)), .redirect)
        XCTAssertEqual(HistoryFormat.tone(summary(status: 500)), .error)
        XCTAssertEqual(HistoryFormat.tone(summary(status: nil)), .error)
        XCTAssertEqual(HistoryFormat.size(summary(size: 0)), "0 B")
        XCTAssertEqual(HistoryFormat.duration(summary(time: 12)), "12 ms")
        XCTAssertEqual(HistoryFormat.duration(summary(time: 1500)), "1.5 s")
        let now = Date(timeIntervalSince1970: 1_700_000_000 + 90)
        XCTAssertEqual(HistoryFormat.age(createdAtMs: 1_700_000_000_000, now: now), "1 min ago")
        XCTAssertEqual(
            HistoryFormat.age(createdAtMs: 1_700_000_000_000, now: now.addingTimeInterval(-60)), "just now")
    }

    func testScratchDropsMaskedHeadersAndKeepsTextBody() {
        let e = entry(
            headers: [
                HeaderPair(name: "Accept", value: "application/json"),
                HeaderPair(name: "Authorization", value: "Bearer ••••••"),
            ], body: "{\"a\":1}")
        let r = HistoryScratch.make(from: e)
        XCTAssertEqual(r.spec.method, "POST")
        XCTAssertEqual(r.spec.url, "https://h/x")
        XCTAssertEqual(r.spec.headers.map(\.name), ["Accept"])
        XCTAssertEqual(r.spec.body, .raw(content: "{\"a\":1}", contentType: nil))
        XCTAssertTrue(r.notice?.contains("Authorization") ?? false)
        XCTAssertFalse(r.spec.authenticated)
    }

    func testScratchEmptyBodyAndNoNotice() {
        let r = HistoryScratch.make(from: entry(headers: [HeaderPair(name: "A", value: "b")]))
        XCTAssertEqual(r.spec.body, .empty)
        XCTAssertNil(r.notice)
    }

    func testScratchSkipsBinaryAndMultipartBodies() {
        let bin = HistoryScratch.make(from: entry(body: "<binary body, 12 bytes>"))
        XCTAssertEqual(bin.spec.body, .empty)
        XCTAssertTrue(bin.notice?.contains("body") ?? false)

        let multi = HistoryScratch.make(
            from: entry(
                headers: [HeaderPair(name: "Content-Type", value: "multipart/form-data; boundary=x")],
                body: "f: <file a.txt, 3 bytes>"))
        XCTAssertEqual(multi.spec.body, .empty)
        XCTAssertTrue(multi.notice?.contains("body") ?? false)
    }
}
